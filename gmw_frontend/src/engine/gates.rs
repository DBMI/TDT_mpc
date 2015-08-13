use protocols::ProtocolDesc;
use types::OblivVar;

pub trait OXor {
    type PD: ProtocolDesc;
    type OType: OblivVar;
    fn o_xor(&self, pd: &Self::PD, other: &Self::OType)->Self::OType;
}

pub trait OAnd {
    type PD: ProtocolDesc;
    type OType: OblivVar;
    fn o_and(&self, pd: &Self::PD, other: &Self::OType)->Self::OType;
}

pub trait OOr {
    type PD: ProtocolDesc;
    type OType: OblivVar;
    fn o_or(&self, pd: &Self::PD, other: &Self::OType)->Self::OType;
}

pub trait ONot {
    type PD: ProtocolDesc;
    type OType: OblivVar;
    fn o_not(&self, pd: &Self::PD)->Self::OType;
    fn o_not_inplace(&mut self, pd: &Self::PD);
}


