use malachite_core_types::SigningProvider;

use super::BaseContext;

#[derive(Clone)]
pub struct BaseSigningProvider {}

impl BaseSigningProvider {
    pub fn new() -> BaseSigningProvider {
        Self {}
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
        todo!()
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
        todo!()
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
