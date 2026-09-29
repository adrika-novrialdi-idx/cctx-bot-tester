package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/clierr"
	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/register"
)

var rootCmd = &cobra.Command{
	Use:           "indodax-sandbox",
	Short:         "Indodax TAPI v2 sandbox bot",
	Long:          "Talk to the Indodax sandbox through CCXT without using production hosts.",
	SilenceErrors: true,
	SilenceUsage:  true,
}

func main() {
	register.RegisterCommands(rootCmd)
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		os.Exit(clierr.ExitCode(err))
	}
}
