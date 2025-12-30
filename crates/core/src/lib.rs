pub mod ids;
pub mod views;

pub type Epoch = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EpochPair {
    pub graph_epoch: Epoch,
    pub manifold_epoch: Epoch,
}
