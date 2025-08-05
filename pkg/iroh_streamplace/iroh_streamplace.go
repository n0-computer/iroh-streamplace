package iroh_streamplace

import (
	iroh "github.com/n0-computer/iroh-streamplace/pkg/iroh_streamplace/generated/iroh_streamplace"
)

// #cgo LDFLAGS: -L../../target/release -liroh_streamplace -lm
// #cgo darwin LDFLAGS: -framework Security
import "C"

func NewSenderEndpoint() *iroh.SenderEndpoint {
	ep := iroh.NewSenderEndpoint()
	return ep
}
