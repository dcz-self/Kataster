use bevy::prelude::*;
use core::fmt;
use crate::util::PredicateContainer;
use super::arena::*;
use super::mob::GenePool;
use super::shooter;


/// Component to tag an entity as only needed in one game state
#[derive(Clone)]
pub enum ForStates<T: 'static> {
    Func(PredicateContainer<T>),
}

impl<T: PartialEq> ForStates<T> {
    pub fn from_func<F: Fn(&T) -> bool + Send + Sync + Clone + 'static>(pred: F)
        -> Self
    {
        ForStates::Func(PredicateContainer::new(pred))// { function: Box::new(pred) } )
    }

    pub fn covers(&self, state: &T) -> bool {
        match self {
            ForStates::Func(pred) => pred.apply(state),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    AI,
    Player,
}

pub type ValidStates = ForStates<GameState>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameState {
    /// Launch game state
    Begin,
    MainMenu,
    Manager,
    Arena(Mode),
    ArenaPause(Mode),
    /// Round summary. Physics goes on, but the action is finished.
    ArenaOver(Mode),
    /// Helper to clean up the arena
    BetweenRounds,
}

impl Default for GameState {
    fn default() -> GameState {
        GameState::Begin
    }
}

impl GameState {
    /// Play is not over yet.
    pub fn is_live_arena(&self) -> bool {
        use super::GameState as S;
        match self {
            S::Arena(_) => true,
            S::ArenaPause(_) => true,
            _ => false,
        }
    }
    pub fn is_arena(&self) -> bool {
        self.arena_mode().is_some()
    }
    pub fn arena_mode(&self) -> Option<Mode> {
        use super::GameState as S;
        match self.clone() {
            S::Arena(mode) => Some(mode),
            S::ArenaPause(mode) => Some(mode),
            S::ArenaOver(mode) => Some(mode),
            _ => None,
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
enum Transition<T: PartialEq + Eq + Copy + fmt::Debug> {
    ExitFor(T),
    Enter(T),
    None,
}

#[derive(Debug)]
pub struct GameStateFsm<T: PartialEq + Eq + Copy + fmt::Debug> {
    current: T,
    next: Transition<T>,
}

impl<T: PartialEq + Eq + Copy + fmt::Debug + Default> GameStateFsm<T> {
    pub fn new(initial: T) -> GameStateFsm<T> {
        GameStateFsm {
            current: Default::default(),
            next: Transition::ExitFor(initial),
        }
    }
    pub fn current(&self) -> &T {
        &self.current
    }
    pub fn is(&self, state: T) -> bool {
        self.current == state
    }

    fn exiting_group(&self, states: &ForStates<T>) -> bool {
        match &self.next {
            Transition::ExitFor(next) => {
                states.covers(&self.current)
                    && !states.covers(next)
            },
            _ => false,
        }
    }

    pub fn entering(&self) -> Option<&T> {
        match &self.next {
            Transition::Enter(next) => Some(next),
            _ => None,
        }
     }

    /// Returns true if entering a state in the group
    /// from a state not in the group
    pub fn entering_group_pred<F: Fn(&T)->bool>(&self, pred: F) -> bool {
        match self.next {
            Transition::Enter(next) => pred(&next) && !pred(&self.current),
            _ => false,
        }
    }
    pub fn transit_to(&mut self, state: T) {
        if self.next != Transition::None {
            eprintln!("Not going to {:?}, transition already in progress", state);
            return;
        }
        self.next = Transition::ExitFor(state);
    }
    /// Called every frame to update the phases of transitions.
    /// A transition requires 2 frames: Exit current, enter next
    pub fn update(&mut self) {
        match self.next.clone() {
            Transition::ExitFor(next) => {
                // We have exited current state, we can enter the new one
                self.next = Transition::Enter(next);
            },
            Transition::Enter(next) => {
                // We have entered the new one it is now current
                self.current = next;
                self.next = Transition::None;
            },
            _ => {},
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
        assert!(fsm.exiting_group(
            &ForStates::<States>::from_func(|s| s == &States::AA)
        ));
    }
}
