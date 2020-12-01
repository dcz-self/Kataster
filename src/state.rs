use bevy::prelude::*;
use core::fmt;
use super::arena::*;
use super::mob::GenePool;
use super::shooter;

/// Component to tag an entity as only needed in one game state
pub enum ForStates<T> {
    Func(Box<dyn Fn(&T) -> bool + Send + Sync>),
}

impl<T: PartialEq> ForStates<T> {
    pub fn from_func<F: Fn(&T) -> bool + Send + Sync + 'static>(pred: F)
        -> Self
    {
        ForStates::Func(Box::new(pred))
    }

    pub fn covers(&self, state: &T) -> bool {
        match self {
            ForStates::Func(pred) => pred(state),
        }
    }
}


pub type ValidStates = ForStates<GameState>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameState {
    /// Launch game state
    Begin,
    MainMenu,
    Manager,
    Arena,
    ArenaPause,
    ArenaOver,
    BetweenRounds,
}

impl Default for GameState {
    fn default() -> GameState {
        GameState::Begin
    }
}

impl GameState {
    pub fn is_live_arena(&self) -> bool {
        use super::GameState as S;
        match self {
            S::Arena => true,
            S::ArenaPause => true,
            _ => false,
        }
    }
    pub fn is_arena(&self) -> bool {
        use super::GameState as S;
        match self {
            S::Arena => true,
            S::ArenaPause => true,
            S::ArenaOver => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct RunState {
    pub gamestate: GameStateFsm<GameState>,
    pub player: Option<Entity>,
    pub arena: Option<Arena>,
    pub score: Option<u32>,
    pub mob_gene_pool: GenePool,
    pub shooter_gene_pool: shooter::GenePool,
}

impl RunState {
    pub fn new(start: GameState) -> RunState {
        RunState {
            gamestate: GameStateFsm::new(start),
            player: None,
            arena: None,
            score: None,
            mob_gene_pool: GenePool::new_eden(),
            shooter_gene_pool: shooter::GenePool::new_eden(),
        }
    }
}

pub fn runstate_fsm(mut runstate: ResMut<RunState>) {
    runstate.gamestate.update();
}

pub fn state_exit_despawn(
    mut commands: Commands,
    runstate: ResMut<RunState>,
    query: Query<(Entity, &ForStates<GameState>)>,
) {
    for (entity, for_states) in &mut query.iter() {
        if runstate.gamestate.exiting_group(for_states) {
            commands.despawn(entity);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum FsmTransition {
    Exit,
    Enter,
    None,
}
#[derive(Debug)]
pub struct GameStateFsm<T: PartialEq + Eq + Copy + fmt::Debug> {
    current: T,
    transition: FsmTransition,
    next: Option<T>,
    prev: Option<T>,
}

impl<T: PartialEq + Eq + Copy + fmt::Debug + Default> GameStateFsm<T> {
    pub fn new(initial: T) -> GameStateFsm<T> {
        GameStateFsm {
            current: Default::default(),
            transition: FsmTransition::Enter,
            next: Some(initial),
            prev: None,
        }
    }
    pub fn current(&self) -> &T {
        &self.current
    }
    pub fn is(&self, state: T) -> bool {
        self.current == state
    }

    fn exiting_group(&self, states: &ForStates<T>) -> bool {
        self.transition == FsmTransition::Exit
            && states.covers(&self.current)
            && !self.next.as_ref()
                    .map(|s| states.covers(s))
                    .unwrap_or(true)
    }

    pub fn entering(&self) -> Option<&T> {
        match self.transition {
            FsmTransition::Enter => self.next.as_ref(),
            _ => None,
        }
     }

    /// Returns true if entering a state in the group
    /// from a state not in the group
    pub fn entering_group(&self, states: &[T]) -> bool {
        self.entering_group_pred(|state| states.contains(state))
    }
    pub fn entering_group_pred<F: Fn(&T)->bool>(&self, pred: F) -> bool {
        self.transition == FsmTransition::Enter
            && self.next.as_ref()
                .map(|next| pred(next))
                .unwrap_or(false)
            && !self.prev.as_ref()
                .map(|prev| pred(prev))
                .unwrap_or(false)
    }
    pub fn transit_to(&mut self, state: T) {
        self.next = Some(state);
        self.transition = FsmTransition::Exit;
    }
    /// Called every frame to update the phases of transitions.
    /// A transition requires 3 frames: Exit current, enter next, current=next
    pub fn update(&mut self) {
        if let Some(next) = self.next {
            match self.transition {
                FsmTransition::Exit => {
                    // We have exited current state, we can enter the new one
                    self.prev = Some(self.current);
                    self.transition = FsmTransition::Enter;
                },
                FsmTransition::Enter => {
                    // We have entered the new one it is now current
                    self.current = next;
                    self.transition = FsmTransition::None;
                    self.next = None;
                },
                _ => {},
            }
            //println!("After Update {:?}", self);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum States {
        AA,
        B,
    }
    
    impl Default for States {
        fn default() -> States {
            States::AA
        }
    }

    #[test]
    pub fn exiting() {
        let mut fsm = GameStateFsm::new(States::AA);
        fsm.update();
        fsm.update();
        fsm.update();
        fsm.transit_to(States::B);
        fsm.update();
        assert!(fsm.exiting_group(
            &ForStates::<States>::from_func(|s| s == &States::AA)
        ));
    }
}
