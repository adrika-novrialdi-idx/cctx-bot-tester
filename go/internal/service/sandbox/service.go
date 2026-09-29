package sandbox

import (
	"fmt"
	"strconv"

	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/adapter/indodax"
	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/clierr"
	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/config"
	ccxt "github.com/ccxt/ccxt/go/v4"
)

const (
	symbol          = "BTC/USDT"
	targetNotional  = 10.0
	maxNotionalUSD  = 25.0
	publicHostLine  = "https://d-idx.com"
	privateHostLine = "https://api.d-idx.com"
)

// Exchange is the Indodax operations required by the sandbox workflow.
type Exchange interface {
	FetchTime() (int64, error)
	FetchBalance() error
	FetchTicker(symbol string) (ccxt.Ticker, error)
	LoadMarkets() error
	CreateOrder(symbol, typeVar, side string, amount, price float64) (string, error)
	CancelOrder(id, symbol string) error
	PriceToPrecision(symbol string, price float64) (float64, error)
	AmountToPrecision(symbol string, amount float64) (float64, error)
}

// Input controls a sandbox run.
type Input struct {
	// Trade enables the limit-buy then cancel path after the read-only checks.
	Trade bool
}

// Result is the stdout lines produced by a successful run.
type Result struct {
	// Lines are the messages to print to stdout, in order.
	Lines []string
}

// Service orchestrates the Indodax sandbox workflow.
type Service interface {
	Run(in Input) (Result, error)
}

type service struct {
	exchange Exchange
}

// NewService builds a sandbox service from credentials and verbosity.
func NewService(creds config.Credentials, verbose bool) (Service, error) {
	client := indodax.NewClient(indodax.Config{
		APIKey:  creds.APIKey,
		Secret:  creds.Secret,
		Verbose: verbose,
	})
	return &service{exchange: client}, nil
}

// NewServiceWithExchange builds a sandbox service with an injected exchange.
func NewServiceWithExchange(exchange Exchange) Service {
	return &service{exchange: exchange}
}

// Run executes the read-only path and optionally the trade path.
func (s *service) Run(in Input) (Result, error) {
	var lines []string

	ms, err := s.exchange.FetchTime()
	if err != nil {
		return Result{}, err
	}
	lines = append(lines, fmt.Sprintf("server time: %d", ms))

	if err := s.exchange.FetchBalance(); err != nil {
		return Result{}, err
	}
	lines = append(lines, fmt.Sprintf("ok public=%s private=%s", publicHostLine, privateHostLine))

	if !in.Trade {
		return Result{Lines: lines}, nil
	}

	tradeLines, err := s.runTrade()
	if err != nil {
		return Result{}, err
	}
	lines = append(lines, tradeLines...)
	return Result{Lines: lines}, nil
}

func (s *service) runTrade() (lines []string, err error) {
	ticker, err := s.exchange.FetchTicker(symbol)
	if err != nil {
		return nil, err
	}
	if ticker.Last == nil || *ticker.Last <= 0 {
		return nil, clierr.New(clierr.ConfigMissing, "ticker last price is missing or not greater than 0")
	}

	if err := s.exchange.LoadMarkets(); err != nil {
		return nil, err
	}

	rawPrice := *ticker.Last * 0.5
	rawAmount := targetNotional / rawPrice

	precisePrice, err := s.exchange.PriceToPrecision(symbol, rawPrice)
	if err != nil {
		return nil, err
	}
	preciseAmount, err := s.exchange.AmountToPrecision(symbol, rawAmount)
	if err != nil {
		return nil, err
	}

	notional := preciseAmount * precisePrice
	if notional >= maxNotionalUSD || preciseAmount == 0 {
		return nil, clierr.New(
			clierr.NotionalRefused,
			fmt.Sprintf("refused: notional %s USD is not below 25", formatNotional(notional)),
		)
	}

	var orderID string
	defer func() {
		if orderID == "" {
			return
		}
		if cerr := s.exchange.CancelOrder(orderID, symbol); cerr != nil {
			err = cerr
			return
		}
		lines = []string{
			fmt.Sprintf("order id: %s", orderID),
			fmt.Sprintf("canceled order id: %s", orderID),
		}
	}()

	orderID, err = s.exchange.CreateOrder(symbol, "limit", "buy", preciseAmount, precisePrice)
	if err != nil {
		orderID = ""
		return nil, err
	}
	return nil, nil
}

func formatNotional(v float64) string {
	return strconv.FormatFloat(v, 'f', -1, 64)
}
