package iroh_streamplace

import (
	"testing"

	"github.com/stretchr/testify/assert"

	iroh "github.com/n0-computer/iroh-streamplace/pkg/iroh_streamplace/generated/iroh_streamplace"
)

type TestHandler struct {
	messages chan []byte
}

func (handler TestHandler) HandleData(node *iroh.PublicKey, data []byte) {
	handler.messages <- data
}

func TestSender(t *testing.T)  {
        sender := iroh.NewSenderEndpoint()

	messages := make(chan []byte, 5)
	handler := TestHandler { messages: messages }

        receiver := iroh.NewReceiverEndpoint(&handler)

        receiverAddr := receiver.NodeAddr()
        receiverId := receiverAddr.NodeId()

        // add peer
        sender.AddPeer(receiverAddr)

        // send a few messages
        for i := range(5) {
		sender.Send(receiverId, []byte{ byte(i), 0, 0, 0 })
        }

         // make sure the receiver got them
	for i := range(5) {
		msg := <- messages
		assert.Equal(t, msg, []byte{byte(i), 0, 0, 0 })
	}
}
