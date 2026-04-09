//! Groth16 circuit definition.
//!
//! See [spec-groth16-circuit.md](../../docs/resources/spec-groth16-circuit.md).

use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// The consensus circuit for proving validator set membership and majority.
#[derive(Clone)]
pub struct ConsensusCircuit {
    // TODO: Add circuit fields
    _placeholder: (),
}

impl ConsensusCircuit {
    /// Create a new circuit with the given witness.
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for ConsensusCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSynthesizer<ark_bls12_381::Fr> for ConsensusCircuit {
    fn generate_constraints(
        self,
        _cs: ConstraintSystemRef<ark_bls12_381::Fr>,
    ) -> Result<(), SynthesisError> {
        // TODO: Implement circuit constraints
        Ok(())
    }
}
