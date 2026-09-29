package clierr

import (
	"errors"
	"fmt"
)

// ErrorCode is a stable machine-readable failure class.
type ErrorCode string

const (
	// ConfigMissing indicates a required credential is empty.
	ConfigMissing ErrorCode = "CONFIG_MISSING"
	// NotionalRefused indicates the computed order notional is not below 25 USD.
	NotionalRefused ErrorCode = "NOTIONAL_REFUSED"
	// Exchange indicates a CCXT or network failure.
	Exchange ErrorCode = "EXCHANGE"
)

// Error is a typed CLI failure with an optional wrapped cause.
type Error struct {
	Code    ErrorCode
	Message string
	Err     error
}

func (e *Error) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("%s: %v", e.Message, e.Err)
	}
	return e.Message
}

// Unwrap returns the wrapped cause.
func (e *Error) Unwrap() error {
	return e.Err
}

// New builds an Error without a wrapped cause.
func New(code ErrorCode, message string) *Error {
	return &Error{Code: code, Message: message}
}

// NewWithErr builds an Error that wraps a cause.
func NewWithErr(code ErrorCode, message string, err error) *Error {
	return &Error{Code: code, Message: message, Err: err}
}

// ExitCode maps a typed error to a process exit status.
func ExitCode(err error) int {
	if err == nil {
		return 0
	}
	var e *Error
	if errors.As(err, &e) {
		switch e.Code {
		case ConfigMissing, NotionalRefused:
			return 2
		}
	}
	return 1
}
