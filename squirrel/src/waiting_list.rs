use std::collections::{HashMap, VecDeque};

use squirrel::{TicketAmount, UserAccount, UserId};

pub(crate) const MAX_USERS_PER_REG_BATCHES: TicketAmount = 1;

pub(crate) type WaitingList = VecDeque<UserId>;
pub(crate) type RegistrationList = HashMap<UserId, UserAccount>;
