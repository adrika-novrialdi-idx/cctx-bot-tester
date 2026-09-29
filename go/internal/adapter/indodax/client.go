package indodax

import (
	"errors"
	"strconv"
	"strings"

	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/clierr"
	ccxt "github.com/ccxt/ccxt/go/v4"
)

const (
	publicHost  = "https://d-idx.com"
	privateHost = "https://api.d-idx.com"
)

// Client wraps the CCXT Indodax exchange for sandbox hosts.
type Client struct {
	exchange *ccxt.Indodax
	apiKey   string
	secret   string
}

// Config configures a sandbox Indodax client.
type Config struct {
	// APIKey is the Indodax API key.
	APIKey string
	// Secret is the Indodax API secret.
	Secret string
	// Verbose enables CCXT verbose logging.
	Verbose bool
}

// NewClient constructs an Indodax client pointed at the sandbox hosts.
func NewClient(cfg Config) *Client {
	exchange := ccxt.NewIndodax(map[string]any{
		"apiKey":          cfg.APIKey,
		"secret":          cfg.Secret,
		"enableRateLimit": true,
		"verbose":         cfg.Verbose,
		"urls": map[string]any{
			"api": map[string]any{
				"public":  publicHost,
				"private": "https://d-idx.com/tapi",
			},
		},
		"options": map[string]any{
			"tapiVersion": "2",
			"sandboxUrl":  privateHost,
		},
	})
	return &Client{
		exchange: exchange,
		apiKey:   cfg.APIKey,
		secret:   cfg.Secret,
	}
}

// FetchTime returns the exchange server time in milliseconds.
func (c *Client) FetchTime() (int64, error) {
	ms, err := c.exchange.FetchTime()
	if err != nil {
		return 0, c.wrap(err)
	}
	return ms, nil
}

// FetchBalance fetches the account balance without returning the payload to callers.
func (c *Client) FetchBalance() error {
	_, err := c.exchange.FetchBalance()
	if err != nil {
		return c.wrap(err)
	}
	return nil
}

// FetchTicker fetches the ticker for the given symbol.
func (c *Client) FetchTicker(symbol string) (ccxt.Ticker, error) {
	ticker, err := c.exchange.FetchTicker(symbol)
	if err != nil {
		return ccxt.Ticker{}, c.wrap(err)
	}
	return ticker, nil
}

// LoadMarkets loads exchange markets required for precision helpers.
func (c *Client) LoadMarkets() error {
	_, err := c.exchange.LoadMarkets()
	if err != nil {
		return c.wrap(err)
	}
	return nil
}

// CreateOrder places an order and returns the order id.
func (c *Client) CreateOrder(symbol, typeVar, side string, amount, price float64) (string, error) {
	order, err := c.exchange.CreateOrder(symbol, typeVar, side, amount, ccxt.WithCreateOrderPrice(price))
	if err != nil {
		return "", c.wrap(err)
	}
	if order.Id == nil || *order.Id == "" {
		return "", clierr.New(clierr.Exchange, "createOrder returned empty order id")
	}
	return *order.Id, nil
}

// CancelOrder cancels an open order by id and symbol.
func (c *Client) CancelOrder(id, symbol string) error {
	_, err := c.exchange.CancelOrder(id, ccxt.WithCancelOrderSymbol(symbol))
	if err != nil {
		return c.wrap(err)
	}
	return nil
}

// PriceToPrecision applies exchange price precision for the symbol.
func (c *Client) PriceToPrecision(symbol string, price float64) (float64, error) {
	raw := c.exchange.PriceToPrecision(symbol, price)
	return parsePrecision(raw, "price")
}

// AmountToPrecision applies exchange amount precision for the symbol.
func (c *Client) AmountToPrecision(symbol string, amount float64) (float64, error) {
	raw := c.exchange.AmountToPrecision(symbol, amount)
	return parsePrecision(raw, "amount")
}

func parsePrecision(raw any, label string) (float64, error) {
	s := strings.TrimSpace(ccxt.ToString(raw))
	if s == "" {
		return 0, clierr.New(clierr.Exchange, "empty "+label+" after precision")
	}
	v, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return 0, clierr.NewWithErr(clierr.Exchange, "failed to parse "+label+" precision", err)
	}
	return v, nil
}

func (c *Client) wrap(err error) error {
	return clierr.NewWithErr(clierr.Exchange, "exchange error", redact(err, c.apiKey, c.secret))
}

func redact(err error, apiKey, secret string) error {
	if err == nil {
		return nil
	}
	msg := err.Error()
	if apiKey != "" {
		msg = strings.ReplaceAll(msg, apiKey, "[REDACTED]")
	}
	if secret != "" {
		msg = strings.ReplaceAll(msg, secret, "[REDACTED]")
	}
	if msg == err.Error() {
		return err
	}
	return errors.New(msg)
}
