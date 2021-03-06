use clarity::Address;
use clarity::Signature;
use failure::Error;
use futures::Future;
use num256::Uint256;

/// An alias for a channel ID in a raw bytes form
pub type ChannelId = [u8; 32];

pub trait PaymentContract {
    fn new_channel(
        &self,
        address0: Address,
        address1: Address,
        balance0: Uint256,
        balance1: Uint256,
        signature_0: Signature,
        signature_1: Signature,
        expiration: Uint256,
        settling_period: Uint256,
    ) -> Box<Future<Item = ChannelId, Error = Error>>;
    fn update_state(
        &self,
        channel_id: ChannelId,
        channel_nonce: Uint256,
        balance_a: Uint256,
        balance_b: Uint256,
        sig_a: Signature,
        sig_b: Signature,
    ) -> Box<Future<Item = (), Error = Error>>;
    fn update_state_with_bounty(
        &self,
        channel_id: ChannelId,
        channel_nonce: Uint256,
        balance_a: Uint256,
        balance_b: Uint256,
        sig_a: Signature,
        sig_b: Signature,
        bounty_amount: Uint256,
        bounty_signature: Signature,
    ) -> Box<Future<Item = (), Error = Error>>;
    fn close_channel_fast(
        &self,
        channel_id: ChannelId,
        channel_nonce: Uint256,
        balance_a: Uint256,
        balance_b: Uint256,
        sig_a: Signature,
        sig_b: Signature,
    ) -> Box<Future<Item = (), Error = Error>>;
    fn close_channel(&self, channel_id: ChannelId) -> Box<Future<Item = (), Error = Error>>;
    fn start_settling_period(
        &self,
        channel_id: ChannelId,
        signature: Signature,
    ) -> Box<Future<Item = (), Error = Error>>;

    fn quick_deposit(&self, value: Uint256) -> Box<Future<Item = (), Error = Error>>;
    fn withdraw(&self, value: Uint256) -> Box<Future<Item = (), Error = Error>>;
    fn redraw(
        &self,
        channel_id: ChannelId,
        channel_nonce: Uint256,
        old_balance_a: Uint256,
        old_balance_b: Uint256,
        new_balance_a: Uint256,
        new_balance_b: Uint256,
        expiration: Uint256,
        sig_a: Signature,
        sig_b: Signature,
    ) -> Box<Future<Item = (), Error = Error>>;
}
