package register

import (
	"github.com/spf13/cobra"

	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/command/sandbox"
)

// RegisterCommands attaches all top-level commands to the root command.
func RegisterCommands(rootCmd *cobra.Command) {
	rootCmd.AddCommand(sandbox.Cmd)
}
