use protocols::ProtocolDesc;

pub trait OblivVar {
    type Input;
    type Output;
    type PD: ProtocolDesc;
    fn feed(pd: &Self::PD, src: Self::Input)->Self::Output;
    fn reveal(&self, pd: &Self::PD)->Self::Input;
    fn is_public(&self)->bool;
}

pub trait OblivBool : OblivVar {
} 

