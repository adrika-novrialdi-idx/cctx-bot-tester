package sandbox

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/config"
	sandboxsvc "github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/service/sandbox"
)

// Cmd is the sandbox run command.
var Cmd = &cobra.Command{
	Use:   "run",
	Short: "Run the Indodax sandbox bot",
	Long:  "Load credentials, call sandbox public and private endpoints, and optionally place then cancel a small limit buy.",
	RunE:  runE,
}

var (
	tradeFlag   bool
	verboseFlag bool
	configFlag  string
)

func init() {
	Cmd.Flags().BoolVar(&tradeFlag, "trade", false, "place and cancel one limit buy after the read-only calls")
	Cmd.Flags().BoolVar(&verboseFlag, "verbose", false, "enable CCXT verbose logging")
	Cmd.Flags().StringVar(&configFlag, "config", "", "path to JSON credentials file (default: config.json)")
}

func runE(cmd *cobra.Command, args []string) error {
	creds, err := config.Load(configFlag)
	if err != nil {
		return err
	}

	svc, err := sandboxsvc.NewService(creds, verboseFlag)
	if err != nil {
		return err
	}

	result, err := svc.Run(sandboxsvc.Input{Trade: tradeFlag})
	if err != nil {
		return err
	}

	for _, line := range result.Lines {
		fmt.Fprintln(cmd.OutOrStdout(), line)
	}
	return nil
}
