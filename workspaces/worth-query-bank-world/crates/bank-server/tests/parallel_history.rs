//! Independent Bank journal rules checked through real authenticated requests.
mod support;
mod parallel_history {
    mod assessment_history;
    mod assessment_trace;
    mod journal_model;
    mod observation;
    mod seeded_world;
    mod transfer_history;
    mod unavailable;
}
