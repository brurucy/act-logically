use crate::engine::storage::RelationStorage;
use datalog_syntax::Program;

pub fn semi_naive_evaluation(
    relation_storage: &mut RelationStorage,
    nonrecursive_delta_program: &Program,
    recursive_delta_program: &Program,
) {
    relation_storage.materialize_nonrecursive_delta_program(&nonrecursive_delta_program);

    loop {
        let previous_non_delta_fact_count = relation_storage.len();

        relation_storage.materialize_recursive_delta_program(&recursive_delta_program);

        let new_non_delta_fact_count = relation_storage.len();

        let new_fact_count = new_non_delta_fact_count - previous_non_delta_fact_count;

        if new_fact_count == 0 {
            return;
        }
    }
}