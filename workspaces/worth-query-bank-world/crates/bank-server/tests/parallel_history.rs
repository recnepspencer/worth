//! Independent Bank journal rules checked through real authenticated requests.
#![recursion_limit = "256"]
mod support;
mod parallel_history {
    mod assessment_history;
    mod assessment_trace;
    mod deadline;
    mod journal_model;
    mod observation;
    mod seeded_world;
    mod transfer_history;
    mod unavailable;
}
