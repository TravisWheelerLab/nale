use crate::align::bounded::structs::row_bound_params::RowBoundParams;
use crate::structs::profile::constants::{
    SPECIAL_B, SPECIAL_C, SPECIAL_E, SPECIAL_J, SPECIAL_LOOP, SPECIAL_MOVE, SPECIAL_N,
};
use crate::structs::{DpMatrix, Profile};

pub fn posterior_bounded(
    profile: &Profile,
    forward_matrix: &DpMatrix,
    backward_matrix: &DpMatrix,
    posterior_matrix: &mut DpMatrix,
    params: &RowBoundParams,
) {
    // let target_length = forward_matrix.target_length;

    let overall_score: f32 = forward_matrix.get_special(params.target_end, SPECIAL_C)
        + profile.special_transition_score(SPECIAL_C, SPECIAL_MOVE);

    let mut denominator: f32;

    posterior_matrix.set_special(params.target_start - 1, SPECIAL_E, 0.0);
    posterior_matrix.set_special(params.target_start - 1, SPECIAL_N, 0.0);
    posterior_matrix.set_special(params.target_start - 1, SPECIAL_J, 0.0);
    posterior_matrix.set_special(params.target_start - 1, SPECIAL_B, 0.0);
    posterior_matrix.set_special(params.target_start - 1, SPECIAL_C, 0.0);

    let profile_start_in_first_row = params.left_row_bounds[params.target_start];
    let profile_end_in_first_row = params.right_row_bounds[params.target_start];

    // for profile_idx in 0..=profile.length {
    for profile_idx in (profile_start_in_first_row - 1)..=profile_end_in_first_row {
        posterior_matrix.set_match(params.target_start - 1, profile_idx, 0.0);
        posterior_matrix.set_insert(params.target_start - 1, profile_idx, 0.0);
        posterior_matrix.set_delete(params.target_start - 1, profile_idx, 0.0);
    }

    // for target_idx in 1..=target_length {
    for target_idx in params.target_start..=params.target_end {
        denominator = 0.0;
        posterior_matrix.set_match(target_idx, params.target_start - 1, 0.0);
        posterior_matrix.set_insert(target_idx, params.target_start - 1, 0.0);
        posterior_matrix.set_delete(target_idx, params.target_start - 1, 0.0);

        let profile_start_in_current_row = params.left_row_bounds[target_idx];
        let profile_end_in_current_row = params.right_row_bounds[target_idx];

        // for profile_idx in 1..profile.length {
        for profile_idx in profile_start_in_current_row..profile_end_in_current_row {
            posterior_matrix.set_match(
                target_idx,
                profile_idx,
                (forward_matrix.get_match(target_idx, profile_idx)
                    + backward_matrix.get_match(target_idx, profile_idx)
                    - overall_score)
                    .exp(),
            );

            denominator += posterior_matrix.get_match(target_idx, profile_idx);

            posterior_matrix.set_insert(
                target_idx,
                profile_idx,
                (forward_matrix.get_insert(target_idx, profile_idx)
                    + backward_matrix.get_insert(target_idx, profile_idx)
                    - overall_score)
                    .exp(),
            );
            denominator += posterior_matrix.get_insert(target_idx, profile_idx);

            posterior_matrix.set_delete(target_idx, profile_idx, 0.0);
        }

        posterior_matrix.set_match(
            target_idx,
            profile_end_in_current_row,
            (forward_matrix.get_match(target_idx, profile_end_in_current_row)
                + backward_matrix.get_match(target_idx, profile_end_in_current_row)
                - overall_score)
                .exp(),
        );

        denominator += posterior_matrix.get_match(target_idx, profile_end_in_current_row);
        posterior_matrix.set_insert(target_idx, profile_end_in_current_row, 0.0);
        posterior_matrix.set_delete(target_idx, profile_end_in_current_row, 0.0);

        posterior_matrix.set_special(target_idx, SPECIAL_E, 0.0);
        posterior_matrix.set_special(
            target_idx,
            SPECIAL_N,
            (forward_matrix.get_special(target_idx - 1, SPECIAL_N)
                + backward_matrix.get_special(target_idx, SPECIAL_N)
                + profile.special_transition_score(SPECIAL_N, SPECIAL_LOOP)
                - overall_score)
                .exp(),
        );

        posterior_matrix.set_special(
            target_idx,
            SPECIAL_J,
            (forward_matrix.get_special(target_idx - 1, SPECIAL_J)
                + backward_matrix.get_special(target_idx, SPECIAL_J)
                + profile.special_transition_score(SPECIAL_J, SPECIAL_LOOP)
                - overall_score)
                .exp(),
        );

        posterior_matrix.set_special(target_idx, SPECIAL_B, 0.0);

        posterior_matrix.set_special(
            target_idx,
            SPECIAL_C,
            (forward_matrix.get_special(target_idx - 1, SPECIAL_C)
                + backward_matrix.get_special(target_idx, SPECIAL_C)
                + profile.special_transition_score(SPECIAL_C, SPECIAL_LOOP)
                - overall_score)
                .exp(),
        );

        denominator += posterior_matrix.get_special(target_idx, SPECIAL_N);
        denominator += posterior_matrix.get_special(target_idx, SPECIAL_J);
        denominator += posterior_matrix.get_special(target_idx, SPECIAL_C);

        denominator = 1.0 / denominator;

        // for profile_idx in 1..profile.length {
        for profile_idx in profile_start_in_current_row..profile_end_in_current_row {
            posterior_matrix.set_match(
                target_idx,
                profile_idx,
                posterior_matrix.get_match(target_idx, profile_idx) * denominator,
            );
            posterior_matrix.set_insert(
                target_idx,
                profile_idx,
                posterior_matrix.get_insert(target_idx, profile_idx) * denominator,
            );
        }
        posterior_matrix.set_match(
            target_idx,
            profile_end_in_current_row,
            posterior_matrix.get_match(target_idx, profile_end_in_current_row) * denominator,
        );
        posterior_matrix.set_special(
            target_idx,
            SPECIAL_N,
            posterior_matrix.get_special(target_idx, SPECIAL_N) * denominator,
        );
        posterior_matrix.set_special(
            target_idx,
            SPECIAL_J,
            posterior_matrix.get_special(target_idx, SPECIAL_J) * denominator,
        );
        posterior_matrix.set_special(
            target_idx,
            SPECIAL_C,
            posterior_matrix.get_special(target_idx, SPECIAL_C) * denominator,
        );
    }
}
