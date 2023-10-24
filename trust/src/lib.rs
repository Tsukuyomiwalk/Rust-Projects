#![forbid(unsafe_code)]
////////////////////////////////////////////////////////////////////////////////

use crate::Decision::{Cheat, Cooperate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}

pub struct Game {
    left_current_score: i32,
    right_current_score: i32,
    left_agent: Box<dyn Agent>,
    right_agent: Box<dyn Agent>,
    last_decision_l: Decision,
    last_decision_r: Decision,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Decision {
    Cooperate,
    Cheat,
}

pub trait Agent {
    fn decide(&mut self, decision: Decision) -> Decision;
}

impl Game {
    pub fn new(left: Box<dyn Agent>, right: Box<dyn Agent>) -> Self {
        Game {
            left_current_score: 0,
            right_current_score: 0,
            left_agent: left,
            right_agent: right,
            last_decision_l: Cooperate,
            last_decision_r: Cooperate,
        }
    }

    pub fn left_score(&self) -> i32 {
        self.left_current_score
    }

    pub fn right_score(&self) -> i32 {
        self.right_current_score
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let left_decision = self.left_agent.decide(self.last_decision_r);
        let right_decision = self.right_agent.decide(self.last_decision_l);

        match (left_decision, right_decision) {
            (Cooperate, Cooperate) => {
                self.last_decision_l = Cooperate;
                self.last_decision_r = Cooperate;
                self.left_current_score += 2;
                self.right_current_score += 2;
                RoundOutcome::BothCooperated
            }
            (Cheat, Cheat) => {
                self.last_decision_l = Cheat;
                self.last_decision_r = Cheat;
                RoundOutcome::BothCheated
            }

            (Cooperate, Cheat) => {
                self.last_decision_l = Cooperate;
                self.last_decision_r = Cheat;
                self.left_current_score -= 1;
                self.right_current_score += 3;
                RoundOutcome::RightCheated
            }
            (Cheat, Cooperate) => {
                self.last_decision_l = Cheat;
                self.last_decision_r = Cooperate;
                self.left_current_score += 3;
                self.right_current_score -= 1;
                RoundOutcome::LeftCheated
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
pub struct CheatingAgent {}

impl Agent for CheatingAgent {
    fn decide(&mut self, _decision: Decision) -> Decision {
        Cheat
    }
}

impl CheatingAgent {
    pub fn new() -> Self {
        CheatingAgent {}
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CooperatingAgent {}

impl Agent for CooperatingAgent {
    fn decide(&mut self, _decision: Decision) -> Decision {
        Cooperate
    }
}

impl CooperatingAgent {
    pub fn new() -> Self {
        CooperatingAgent {}
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct GrudgerAgent {
    cheated: bool,
}

impl GrudgerAgent {
    pub fn new() -> Self {
        GrudgerAgent { cheated: false }
    }
}

impl Agent for GrudgerAgent {
    fn decide(&mut self, decision: Decision) -> Decision {
        if decision == Cheat {
            self.cheated = true
        }
        if self.cheated {
            Cheat
        } else {
            Cooperate
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////

pub struct CopycatAgent {
    last_decision: Decision,
}

impl CopycatAgent {
    pub fn new() -> Self {
        CopycatAgent {
            last_decision: Cooperate,
        }
    }
}

impl Agent for CopycatAgent {
    fn decide(&mut self, decision: Decision) -> Decision {
        self.last_decision = decision;
        match self.last_decision {
            Cooperate => Cooperate,
            Cheat => Cheat,
        }
    }
}
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////

//detective
pub struct DetectiveAgent {
    past_decisions: Vec<Decision>,
    opponent_cheated: i32,
    last_decision: Decision,
}

impl DetectiveAgent {
    pub fn new() -> Self {
        DetectiveAgent {
            past_decisions: vec![Cooperate, Cooperate, Cheat, Cooperate],
            opponent_cheated: 0,
            last_decision: Cooperate,
        }
    }
}

impl Agent for DetectiveAgent {
    fn decide(&mut self, decision: Decision) -> Decision {
        if self.last_decision == Cheat {
            self.opponent_cheated += 1;
        }
        self.last_decision = decision;
        match self.past_decisions.pop() {
            Some(decision) => decision,
            None => {
                if self.opponent_cheated == 0 {
                    Cheat
                } else {
                    self.last_decision
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Default for CopycatAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CooperatingAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CheatingAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GrudgerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DetectiveAgent {
    fn default() -> Self {
        Self::new()
    }
}
