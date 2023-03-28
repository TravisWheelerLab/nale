use crate::align::bounded::structs::RowBoundParams;
use crate::max_f32;
use crate::structs::dp_matrix::DpMatrix;
use crate::structs::Profile;
use crate::timing::time;

#[funci::timed(timer = time)]
pub fn optimal_accuracy_bounded(
    profile: &Profile,
    posterior_matrix: &impl DpMatrix,
    optimal_matrix: &mut impl DpMatrix,
    params: &RowBoundParams,
) {
    let end_score: f32 = 1.0;

    // initialization of the zero row
    optimal_matrix.set_special(params.target_start - 1, Profile::SPECIAL_N, 0.0);
    optimal_matrix.set_special(params.target_start - 1, Profile::SPECIAL_B, 0.0);
    optimal_matrix.set_special(params.target_start - 1, Profile::SPECIAL_E, -f32::INFINITY);
    optimal_matrix.set_special(params.target_start - 1, Profile::SPECIAL_C, -f32::INFINITY);
    optimal_matrix.set_special(params.target_start - 1, Profile::SPECIAL_J, -f32::INFINITY);

    let profile_start_in_first_row = params.left_row_bounds[params.target_start];
    let profile_end_in_first_row = params.right_row_bounds[params.target_start];

    // for profile_idx in 0..=profile.length {
    for profile_idx in (profile_start_in_first_row - 1)..=profile_end_in_first_row {
        optimal_matrix.set_match(params.target_start - 1, profile_idx, -f32::INFINITY);
        optimal_matrix.set_insert(params.target_start - 1, profile_idx, -f32::INFINITY);
        optimal_matrix.set_delete(params.target_start - 1, profile_idx, -f32::INFINITY);
    }

    // for i in 1..=posterior_matrix.target_length {
    for target_idx in params.target_start..=params.target_end {
        let profile_start_in_current_row = params.left_row_bounds[target_idx];
        let profile_end_in_current_row = params.right_row_bounds[target_idx];

        optimal_matrix.set_match(target_idx, profile_start_in_current_row - 1, -f32::INFINITY);
        optimal_matrix.set_insert(target_idx, profile_start_in_current_row - 1, -f32::INFINITY);
        optimal_matrix.set_delete(target_idx, profile_start_in_current_row - 1, -f32::INFINITY);
        optimal_matrix.set_special(target_idx, Profile::SPECIAL_E, -f32::INFINITY);

        // for profile_idx in 1..profile.length {
        for profile_idx in profile_start_in_current_row..profile_end_in_current_row {
            optimal_matrix.set_match(
                target_idx,
                profile_idx,
                max_f32!(
                    profile
                        .transition_score_delta(Profile::PROFILE_MATCH_TO_MATCH, profile_idx - 1)
                        * (optimal_matrix.get_match(target_idx - 1, profile_idx - 1)
                            + posterior_matrix.get_match(target_idx, profile_idx)),
                    profile
                        .transition_score_delta(Profile::PROFILE_INSERT_TO_MATCH, profile_idx - 1)
                        * (optimal_matrix.get_insert(target_idx - 1, profile_idx - 1)
                            + posterior_matrix.get_match(target_idx, profile_idx)),
                    profile
                        .transition_score_delta(Profile::PROFILE_DELETE_TO_MATCH, profile_idx - 1)
                        * (optimal_matrix.get_delete(target_idx - 1, profile_idx - 1)
                            + posterior_matrix.get_match(target_idx, profile_idx)),
                    profile
                        .transition_score_delta(Profile::PROFILE_BEGIN_TO_MATCH, profile_idx - 1)
                        * (optimal_matrix.get_special(target_idx - 1, Profile::SPECIAL_B)
                            + posterior_matrix.get_match(target_idx, profile_idx))
                ),
            );

            optimal_matrix.set_special(
                target_idx,
                Profile::SPECIAL_E,
                max_f32!(
                    optimal_matrix.get_special(target_idx, Profile::SPECIAL_E),
                    optimal_matrix.get_match(target_idx, profile_idx) * end_score
                ),
            );

            optimal_matrix.set_insert(
                target_idx,
                profile_idx,
                max_f32!(
                    profile.transition_score_delta(Profile::PROFILE_MATCH_TO_INSERT, profile_idx)
                        * (optimal_matrix.get_match(target_idx - 1, profile_idx)
                            + posterior_matrix.get_insert(target_idx, profile_idx)),
                    profile.transition_score_delta(Profile::PROFILE_INSERT_TO_INSERT, profile_idx)
                        * (optimal_matrix.get_insert(target_idx - 1, profile_idx)
                            + posterior_matrix.get_insert(target_idx, profile_idx))
                ),
            );

            optimal_matrix.set_delete(
                target_idx,
                profile_idx,
                max_f32!(
                    profile
                        .transition_score_delta(Profile::PROFILE_MATCH_TO_DELETE, profile_idx - 1)
                        * optimal_matrix.get_match(target_idx, profile_idx - 1),
                    profile
                        .transition_score_delta(Profile::PROFILE_DELETE_TO_DELETE, profile_idx - 1)
                        * optimal_matrix.get_delete(target_idx, profile_idx - 1)
                ),
            );
        }

        optimal_matrix.set_match(
            target_idx,
            profile_end_in_current_row,
            max_f32!(
                profile.transition_score_delta(
                    Profile::PROFILE_MATCH_TO_MATCH,
                    profile_end_in_current_row - 1
                ) * (optimal_matrix.get_match(target_idx - 1, profile_end_in_current_row - 1)
                    + posterior_matrix.get_match(target_idx, profile_end_in_current_row)),
                profile.transition_score_delta(
                    Profile::PROFILE_INSERT_TO_MATCH,
                    profile_end_in_current_row - 1
                ) * (optimal_matrix.get_insert(target_idx - 1, profile_end_in_current_row - 1)
                    + posterior_matrix.get_match(target_idx, profile_end_in_current_row)),
                profile.transition_score_delta(
                    Profile::PROFILE_DELETE_TO_MATCH,
                    profile_end_in_current_row - 1
                ) * (optimal_matrix.get_delete(target_idx - 1, profile_end_in_current_row - 1)
                    + posterior_matrix.get_match(target_idx, profile_end_in_current_row)),
                profile.transition_score_delta(
                    Profile::PROFILE_BEGIN_TO_MATCH,
                    profile_end_in_current_row - 1
                ) * (optimal_matrix.get_special(target_idx - 1, Profile::SPECIAL_B)
                    + posterior_matrix.get_match(target_idx, profile_end_in_current_row))
            ),
        );

        optimal_matrix.set_delete(
            target_idx,
            profile_end_in_current_row,
            max_f32!(
                profile.transition_score_delta(
                    Profile::PROFILE_MATCH_TO_DELETE,
                    profile_end_in_current_row - 1
                ) * optimal_matrix.get_match(target_idx, profile_end_in_current_row - 1),
                profile.transition_score_delta(
                    Profile::PROFILE_DELETE_TO_DELETE,
                    profile_end_in_current_row - 1
                ) * optimal_matrix.get_delete(target_idx, profile_end_in_current_row - 1)
            ),
        );

        // a comment from hmmer:
        //   now the special states; it's important that E is already done, and B is done after N,J
        optimal_matrix.set_special(
            target_idx,
            Profile::SPECIAL_E,
            max_f32!(
                optimal_matrix.get_special(target_idx, Profile::SPECIAL_E),
                optimal_matrix.get_match(target_idx, profile_end_in_current_row),
                optimal_matrix.get_delete(target_idx, profile_end_in_current_row)
            ),
        );

        optimal_matrix.set_special(
            target_idx,
            Profile::SPECIAL_J,
            max_f32!(
                profile.special_transition_score_delta(Profile::SPECIAL_J, Profile::SPECIAL_LOOP)
                    * (optimal_matrix.get_special(target_idx - 1, Profile::SPECIAL_J)
                        + posterior_matrix.get_special(target_idx, Profile::SPECIAL_J)),
                profile.special_transition_score_delta(Profile::SPECIAL_E, Profile::SPECIAL_LOOP)
                    * optimal_matrix.get_special(target_idx, Profile::SPECIAL_E)
            ),
        );

        optimal_matrix.set_special(
            target_idx,
            Profile::SPECIAL_C,
            max_f32!(
                profile.special_transition_score_delta(Profile::SPECIAL_C, Profile::SPECIAL_LOOP)
                    * (optimal_matrix.get_special(target_idx - 1, Profile::SPECIAL_C)
                        + posterior_matrix.get_special(target_idx, Profile::SPECIAL_C)),
                profile.special_transition_score_delta(Profile::SPECIAL_E, Profile::SPECIAL_MOVE)
                    * optimal_matrix.get_special(target_idx, Profile::SPECIAL_E)
            ),
        );

        optimal_matrix.set_special(
            target_idx,
            Profile::SPECIAL_N,
            profile.special_transition_score_delta(Profile::SPECIAL_N, Profile::SPECIAL_LOOP)
                * (optimal_matrix.get_special(target_idx - 1, Profile::SPECIAL_N)
                    + posterior_matrix.get_special(target_idx, Profile::SPECIAL_N)),
        );

        optimal_matrix.set_special(
            target_idx,
            Profile::SPECIAL_B,
            max_f32!(
                profile.special_transition_score_delta(Profile::SPECIAL_N, Profile::SPECIAL_MOVE)
                    * optimal_matrix.get_special(target_idx, Profile::SPECIAL_N),
                profile.special_transition_score_delta(Profile::SPECIAL_J, Profile::SPECIAL_MOVE)
                    * optimal_matrix.get_special(target_idx, Profile::SPECIAL_J)
            ),
        );
    }
}
