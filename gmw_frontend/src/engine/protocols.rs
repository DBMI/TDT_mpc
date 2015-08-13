use circuit::{CircuitDesc, Output}; 

pub trait ProtocolDesc {
    type VarType;
    type CDescType: CircuitDesc;
    fn new(party: u32)->Self;
    fn get_party(&self)->u32;
    fn exec_circuit(&self, circuit: &Self::CDescType)->
        Vec<(Output, Self::VarType)>;
}
