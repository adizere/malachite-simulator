use rand::rngs::OsRng;
use tracing::debug;

use malachite_core_types::{Extension, SignedMessage, SigningProvider, Vote};

use super::{
    signing_scheme::{PrivateKey, PublicKey},
    value::BaseValue,
    vote::BaseVote,
    BaseContext,
};
use crate::context::signing_scheme::Ed25519;

#[derive(Clone)]
pub struct BaseSigningProvider {
    private_key: PrivateKey,
}

impl BaseSigningProvider {
    pub fn new() -> BaseSigningProvider {
        let csprng = OsRng;
        let signing_key = Ed25519::generate_keypair(csprng);

        debug!(public_key = ?signing_key.public_key(), "created new signing provider");

        Self {
            private_key: signing_key,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.private_key.public_key()
    }

    pub fn sign_vote_extended(
        &self,
        vote: BaseVote,
        ex: Option<BaseValue>,
    ) -> SignedMessage<BaseContext, BaseVote> {
        let vote = match ex {
            Some(value) => {
                let extension = Extension::new(value.to_bytes());
                let ext_signature = self.private_key.sign(&extension.data);
                let vote_wext = vote.extend(SignedMessage::new(extension, ext_signature));
                println!(
                    "added ext to {:?} at peer {}: {:?}",
                    vote_wext.vote_type, vote_wext.voter, vote_wext.extension
                );

                vote_wext
            }
            None => vote,
        };

        self.sign_vote(vote)
    }
}

#[allow(unused)]
impl SigningProvider<BaseContext> for BaseSigningProvider {
    fn sign_vote(
        &self,
        vote: <BaseContext as malachite_core_types::Context>::Vote,
    ) -> malachite_core_types::SignedMessage<
        BaseContext,
        <BaseContext as malachite_core_types::Context>::Vote,
    > {
        let signature = self.private_key.sign(&vote.to_bytes());
        SignedMessage::new(vote, signature)
    }

    fn verify_signed_vote(
        &self,
        vote: &<BaseContext as malachite_core_types::Context>::Vote,
        signature: &malachite_core_types::Signature<BaseContext>,
        public_key: &malachite_core_types::PublicKey<BaseContext>,
    ) -> bool {
        todo!()
    }

    fn sign_proposal(
        &self,
        proposal: <BaseContext as malachite_core_types::Context>::Proposal,
    ) -> malachite_core_types::SignedMessage<
        BaseContext,
        <BaseContext as malachite_core_types::Context>::Proposal,
    > {
        let signature = self.private_key.sign(&proposal.to_bytes());
        SignedMessage::new(proposal, signature)
    }

    fn verify_signed_proposal(
        &self,
        proposal: &<BaseContext as malachite_core_types::Context>::Proposal,
        signature: &malachite_core_types::Signature<BaseContext>,
        public_key: &malachite_core_types::PublicKey<BaseContext>,
    ) -> bool {
        todo!()
    }

    fn sign_proposal_part(
        &self,
        proposal_part: <BaseContext as malachite_core_types::Context>::ProposalPart,
    ) -> malachite_core_types::SignedMessage<
        BaseContext,
        <BaseContext as malachite_core_types::Context>::ProposalPart,
    > {
        todo!()
    }

    fn verify_signed_proposal_part(
        &self,
        proposal_part: &<BaseContext as malachite_core_types::Context>::ProposalPart,
        signature: &malachite_core_types::Signature<BaseContext>,
        public_key: &malachite_core_types::PublicKey<BaseContext>,
    ) -> bool {
        todo!()
    }

    fn verify_commit_signature(
        &self,
        certificate: &malachite_core_types::CommitCertificate<BaseContext>,
        commit_sig: &malachite_core_types::CommitSignature<BaseContext>,
        validator: &<BaseContext as malachite_core_types::Context>::Validator,
    ) -> Result<
        malachite_core_types::VotingPower,
        malachite_core_types::CertificateError<BaseContext>,
    > {
        todo!()
    }
}
