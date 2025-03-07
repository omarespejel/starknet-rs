use crate::Call;

use async_trait::async_trait;
use starknet_core::types::{
    contract_artifact::{CompressProgramError, ComputeClassHashError},
    BlockId, ContractArtifact, FieldElement,
};
use starknet_providers::Provider;
use std::{error::Error, sync::Arc};

mod declaration;
mod execution;

/// The standard StarkNet account contract interface. It makes no assumption about the underlying
/// signer or provider. Account implementations that come with an active connection to the network
/// should also implement [ConnectedAccount] for useful functionalities like estimating fees and
/// sending transactions.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Account: Sized {
    type SignError: Error;

    fn address(&self) -> FieldElement;

    fn chain_id(&self) -> FieldElement;

    async fn sign_execution(
        &self,
        execution: &RawExecution,
    ) -> Result<Vec<FieldElement>, Self::SignError>;

    async fn sign_declaration(
        &self,
        declaration: &RawDeclaration,
    ) -> Result<Vec<FieldElement>, Self::SignError>;

    fn execute(&self, calls: Vec<Call>) -> Execution<Self> {
        Execution::new(calls, self)
    }

    fn declare(&self, contract_class: Arc<ContractArtifact>) -> Declaration<Self> {
        Declaration::new(contract_class, self)
    }
}

/// An [Account] implementation that also comes with a [Provider]. Functionalities that require a
/// connection to the sequencer or node are offloaded to this trait to keep the base [Account]
/// clean and flexible.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ConnectedAccount: Account {
    type Provider: Provider + Sync;

    fn provider(&self) -> &Self::Provider;

    /// Block ID to use when checking nonce and estimating fees.
    fn block_id(&self) -> BlockId {
        BlockId::Latest
    }

    async fn get_nonce(&self) -> Result<FieldElement, <Self::Provider as Provider>::Error> {
        self.provider()
            .get_nonce(self.address(), self.block_id())
            .await
    }
}

/// An intermediate type allowing users to optionally specify `nonce` and/or `max_fee`.
#[derive(Debug)]
pub struct Execution<'a, A> {
    account: &'a A,
    calls: Vec<Call>,
    nonce: Option<FieldElement>,
    max_fee: Option<FieldElement>,
    fee_estimate_multiplier: f64,
}

/// An intermediate type allowing users to optionally specify `nonce` and/or `max_fee`.
#[derive(Debug)]
pub struct Declaration<'a, A> {
    account: &'a A,
    contract_class: Arc<ContractArtifact>,
    nonce: Option<FieldElement>,
    max_fee: Option<FieldElement>,
    fee_estimate_multiplier: f64,
}

/// [Execution] but with `nonce` and `max_fee` already determined.
#[derive(Debug)]
pub struct RawExecution {
    calls: Vec<Call>,
    nonce: FieldElement,
    max_fee: FieldElement,
}

/// [Declaration] but with `nonce` and `max_fee` already determined.
#[derive(Debug)]
pub struct RawDeclaration {
    contract_class: Arc<ContractArtifact>,
    nonce: FieldElement,
    max_fee: FieldElement,
}

/// [RawExecution] but with an account associated.
#[derive(Debug)]
pub struct PreparedExecution<'a, A> {
    account: &'a A,
    inner: RawExecution,
}

/// [RawExecution] but with an account associated.
#[derive(Debug)]
pub struct PreparedDeclaration<'a, A> {
    account: &'a A,
    inner: RawDeclaration,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError<S, P> {
    #[error(transparent)]
    Signing(S),
    #[error(transparent)]
    Provider(P),
    #[error(transparent)]
    ClassHashCalculation(ComputeClassHashError),
    #[error(transparent)]
    ClassCompression(CompressProgramError),
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<A> Account for &A
where
    A: Account + Sync,
{
    type SignError = A::SignError;

    fn address(&self) -> FieldElement {
        (*self).address()
    }

    fn chain_id(&self) -> FieldElement {
        (*self).chain_id()
    }

    async fn sign_execution(
        &self,
        execution: &RawExecution,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        (*self).sign_execution(execution).await
    }

    async fn sign_declaration(
        &self,
        declaration: &RawDeclaration,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        (*self).sign_declaration(declaration).await
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<A> Account for Box<A>
where
    A: Account + Sync + Send,
{
    type SignError = A::SignError;

    fn address(&self) -> FieldElement {
        self.as_ref().address()
    }

    fn chain_id(&self) -> FieldElement {
        self.as_ref().chain_id()
    }

    async fn sign_execution(
        &self,
        execution: &RawExecution,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        self.as_ref().sign_execution(execution).await
    }

    async fn sign_declaration(
        &self,
        declaration: &RawDeclaration,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        self.as_ref().sign_declaration(declaration).await
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<A> Account for Arc<A>
where
    A: Account + Sync + Send,
{
    type SignError = A::SignError;

    fn address(&self) -> FieldElement {
        self.as_ref().address()
    }

    fn chain_id(&self) -> FieldElement {
        self.as_ref().chain_id()
    }

    async fn sign_execution(
        &self,
        execution: &RawExecution,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        self.as_ref().sign_execution(execution).await
    }

    async fn sign_declaration(
        &self,
        declaration: &RawDeclaration,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        self.as_ref().sign_declaration(declaration).await
    }
}

impl<A> ConnectedAccount for &A
where
    A: ConnectedAccount + Sync,
{
    type Provider = A::Provider;

    fn provider(&self) -> &Self::Provider {
        (*self).provider()
    }
}

impl<A> ConnectedAccount for Box<A>
where
    A: ConnectedAccount + Sync + Send,
{
    type Provider = A::Provider;

    fn provider(&self) -> &Self::Provider {
        self.as_ref().provider()
    }
}

impl<A> ConnectedAccount for Arc<A>
where
    A: ConnectedAccount + Sync + Send,
{
    type Provider = A::Provider;

    fn provider(&self) -> &Self::Provider {
        self.as_ref().provider()
    }
}
