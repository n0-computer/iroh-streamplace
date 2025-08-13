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

func TestBasicRoundtrip(t *testing.T) {
	ep1, err := iroh.NewEndpoint()
	assert.Nil(t, err)
	sender, err := iroh.NewSender(ep1)
	assert.Nil(t, err)

	messages := make(chan []byte, 5)
	handler := TestHandler{messages: messages}

	ep2, err := iroh.NewEndpoint()
	assert.Nil(t, err)
	receiver, err := iroh.NewReceiver(ep2, &handler)
	assert.Nil(t, err)

	receiverAddr := receiver.NodeAddr()
	receiverId := receiverAddr.NodeId()

	// add peer
	err = sender.AddPeer(receiverAddr)
	assert.Nil(t, err)

	// send a few messages
	for i := range 5 {
		err = sender.Send(receiverId, []byte{byte(i), 0, 0, 0})
		assert.Nil(t, err)
	}

	// make sure the receiver got them
	for i := range 5 {
		msg := <-messages
		assert.Equal(t, msg, []byte{byte(i), 0, 0, 0})
	}
}
