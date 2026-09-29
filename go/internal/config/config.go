package config

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/adrika-novrialdi-idx/cctx-bot-tester/go/internal/clierr"
)

// Credentials holds Indodax API credentials.
type Credentials struct {
	// APIKey is the Indodax API key.
	APIKey string `json:"apiKey"`
	// Secret is the Indodax API secret.
	Secret string `json:"secret"`
}

type fileConfig struct {
	APIKey string `json:"apiKey"`
	Secret string `json:"secret"`
}

// Load reads credentials from a JSON file, then applies non-empty environment overrides.
func Load(path string) (Credentials, error) {
	if path == "" {
		path = "config.json"
	}

	var creds Credentials
	data, err := os.ReadFile(path)
	if err != nil {
		if !os.IsNotExist(err) {
			return Credentials{}, clierr.NewWithErr(clierr.ConfigMissing, fmt.Sprintf("failed to read config %s", path), err)
		}
	} else {
		var file fileConfig
		if err := json.Unmarshal(data, &file); err != nil {
			return Credentials{}, clierr.NewWithErr(clierr.ConfigMissing, fmt.Sprintf("failed to parse config %s", path), err)
		}
		creds.APIKey = file.APIKey
		creds.Secret = file.Secret
	}

	if v := os.Getenv("INDODAX_APIKEY"); v != "" {
		creds.APIKey = v
	}
	if v := os.Getenv("INDODAX_SECRET"); v != "" {
		creds.Secret = v
	}

	if creds.APIKey == "" {
		return Credentials{}, clierr.New(clierr.ConfigMissing, "missing INDODAX_APIKEY")
	}
	if creds.Secret == "" {
		return Credentials{}, clierr.New(clierr.ConfigMissing, "missing INDODAX_SECRET")
	}

	return creds, nil
}
