use crate::{State, StateCacheType, StateRegistry};

struct StateA;
impl State for StateA {
    type Parameter = ();

    fn create(_states: &StateRegistry, _params: Self::Parameter) -> anyhow::Result<Self> {
        println!("State A created");
        Ok(Self)
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

struct StateB;
impl State for StateB {
    type Parameter = ();

    fn create(_states: &StateRegistry, _params: Self::Parameter) -> anyhow::Result<Self> {
        println!("State B created");
        Ok(Self)
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

struct StateC;
impl State for StateC {
    type Parameter = u64;

    fn create(states: &StateRegistry, params: Self::Parameter) -> anyhow::Result<Self> {
        assert!(states.resolve::<StateA>(()).is_ok());
        println!("State C({}) created", params);
        assert!(states.resolve::<StateB>(()).is_ok());
        if params == 1 {
            assert!(states.resolve::<StateC>(1).is_err());
        } else {
            assert!(states.resolve::<StateC>(1).is_ok());
        }
        Ok(Self)
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

#[test]
fn test_creation_0() {
    let states = StateRegistry::new(10);
    assert!(states.resolve::<StateA>(()).is_ok());
}

#[test]
fn test_creation_1() {
    let states = StateRegistry::new(4);
    assert!(states.resolve::<StateC>(0).is_ok());
}

#[test]
fn test_expire() {
    let mut states = StateRegistry::new(2);
    assert!(states.resolve::<StateA>(()).is_ok());
    assert!(states.resolve::<StateB>(()).is_ok());
    states.invalidate_states();
    assert!(states.get::<StateA>(()).is_none());
    assert!(states.get::<StateB>(()).is_some());
    assert!(states.resolve::<StateA>(()).is_ok());
    assert!(states.resolve::<StateB>(()).is_ok());
    assert!(states.get::<StateA>(()).is_some());
    assert!(states.get::<StateB>(()).is_some());
} 