use crate::align::bounded::structs::RowBoundParams;
use crate::log_sum;
use crate::structs::dp_matrix::DpMatrix;
use crate::structs::{Profile, Sequence};
use crate::timing::time;
use crate::util::log_add;

#[funci::timed(timer = time)]
pub fn backward_bounded(
    profile: &Profile,
    target: &Sequence,
    // dp_matrix: &mut DpMatrix3D,
    dp_matrix: &mut impl DpMatrix,
    params: &RowBoundParams,
) {
    let end_score: f32 = 0.0;
    //  M   s_D
    //  s_I s_M
    //
    //  I    -
    //  s_I s_M
    //
    //  D   s_D
    //  -   s_M
    //

    // C:   a cell in the cloud
    // L:   a cell in the last *row* of the cloud
    // [p]: a padding cell
    //
    //  .........................
    //  .........................
    //  ....  C  C  C  C  C  C  C [p]
    //  ....  C  C  C  C  C  C [p][p] <-  we need different amounts of
    //  ....  C  C  C  C  C  C [p]    <-  padding depending on whether
    //  ....  C  C  C  C  C [p][p]    <-  or not the rows are offset
    //  ....  L  L  L  L [p][p]
    //        ^  ^  ^  ^
    //       the last row defined in the row bounds
    //       is initialized with special conditions

    dp_matrix.set_special(params.target_start, Profile::SPECIAL_J, -f32::INFINITY);
    dp_matrix.set_special(params.target_start, Profile::SPECIAL_B, -f32::INFINITY);
    dp_matrix.set_special(params.target_start, Profile::SPECIAL_N, -f32::INFINITY);

    dp_matrix.set_special(
        params.target_end,
        Profile::SPECIAL_C,
        profile.special_transition_score(Profile::SPECIAL_C, Profile::SPECIAL_MOVE),
    );

    dp_matrix.set_special(
        params.target_end,
        Profile::SPECIAL_E,
        profile.special_transition_score(Profile::SPECIAL_C, Profile::SPECIAL_MOVE)
            + profile.special_transition_score(Profile::SPECIAL_E, Profile::SPECIAL_MOVE),
    );

    let profile_start_on_last_row = params.left_row_bounds[params.target_end];
    let profile_end_on_last_row = params.right_row_bounds[params.target_end];

    // C: dp matrix cell
    // L: cell in the last row
    // *: the last cell on the last row
    //
    //     bounded                       full
    //  C  C  C  C  C  -            F  F  F  F  F  F
    //  C  C  C  C  -  -            F  F  F  F  F  F
    //  C  C  C  C  -  -   <-vs->   F  F  F  F  F  F
    //  C  C  C  -  -  -            F  F  F  F  F  F
    //  L  *  -  -  -  -            L  L  L  L  L  *

    dp_matrix.set_match(
        params.target_end,
        profile_end_on_last_row,
        dp_matrix.get_special(params.target_end, Profile::SPECIAL_E),
    );

    dp_matrix.set_delete(
        params.target_end,
        profile_end_on_last_row,
        dp_matrix.get_special(params.target_end, Profile::SPECIAL_E),
    );

    dp_matrix.set_insert(params.target_end, profile_end_on_last_row, -f32::INFINITY);

    // C: dp matrix cell
    // L: cell in the last row
    // *: the last cell on the last row
    //
    //     bounded                       full
    //  C  C  C  C  C  -            F  F  F  F  F  F
    //  C  C  C  C  -  -   <-vs->   F  F  F  F  F  F
    //  C  C  C  -  -  -            F  F  F  F  F  F
    //  L  *  -  -  -  -            F  F  F  F  F  F
    //  -  -  -  -  -  -            L  L  L  L  L  *
    //
    // this loops over the last row, setting all of the <L> cells, excluding the last cell <*>

    // for profile_idx in (1..profile.length).rev() {
    for profile_idx in (profile_start_on_last_row..profile_end_on_last_row).rev() {
        dp_matrix.set_match(
            params.target_end,
            profile_idx,
            log_sum!(
                dp_matrix.get_special(params.target_end, Profile::SPECIAL_E) + end_score,
                dp_matrix.get_delete(params.target_end, profile_idx + 1)
                    + profile.transition_score(Profile::PROFILE_MATCH_TO_DELETE, profile_idx)
            ),
        );

        dp_matrix.set_insert(params.target_end, profile_idx, -f32::INFINITY);
        dp_matrix.set_delete(
            params.target_end,
            profile_idx,
            log_sum!(
                dp_matrix.get_special(params.target_end, Profile::SPECIAL_E) + end_score,
                dp_matrix.get_delete(params.target_end, profile_idx + 1)
                    + profile.transition_score(Profile::PROFILE_DELETE_TO_DELETE, profile_idx)
            ),
        );
    }

    // main recursion
    // for target_idx in (1..target.length).rev() {
    for target_idx in (params.target_start..params.target_end).rev() {
        let current_residue = target.digital_bytes[target_idx + 1] as usize;
        //            Backward matrix             B state
        // ... .   ...  .  .  .  .  .  .  .  .       .
        // ... .   ...  .  .  .  .  .  .  .  .       .
        // ... .   ...  .  .  .  .  .  .  .  .       .
        // ... C   ...  C  C  C  C  C  C  C  -       -
        // ... C   ...  C  C  C  C  C  C  C  -       -
        // ... C   ...  C  C  C  C  C  C  C  -       -
        // ... C   ...  C  C  C  C  C  C  -  -      B_t = M_p * tsc(B->M) * msc(T_t) // init
        //    M_p  ...  C  C  C  C  C  -  -  -      B_p // the previous B state
        //              .  .  .  .  .  .  .  .       .
        //              .  .  .  .  .  .  .  .       .
        //              .  .  .  .  .  .  .  .       .
        //              L  L  L  L  -  -  -  -      B_L  // the first B state
        //              -  -  -  -  -  -  -  -
        //

        let profile_start_on_current_row = params.left_row_bounds[target_idx];
        let profile_end_on_current_row = params.right_row_bounds[target_idx];

        // TODO: I don't think we need to do this as long as the B state is initialized with -inf
        //       if we do need to do this for some reason, we need to change the bounds of the next loop
        dp_matrix.set_special(
            target_idx,
            Profile::SPECIAL_B,
            dp_matrix.get_match(target_idx + 1, 1)
                + profile
                    .transition_score(Profile::PROFILE_BEGIN_TO_MATCH, profile_start_on_current_row - 1)
                + profile.match_score(current_residue, profile_start_on_current_row),
        );

        // this loops over the cells in the current row, incrementally adding to the B state
        // it depends on the match scores in the next row being computed first

        // for profile_idx in 2..=profile.length {
        for profile_idx in (profile_start_on_current_row + 1)..=profile_end_on_current_row {
            dp_matrix.set_special(
                target_idx,
                Profile::SPECIAL_B,
                log_sum!(
                    dp_matrix.get_special(target_idx, Profile::SPECIAL_B),
                    dp_matrix.get_match(target_idx + 1, profile_idx)
                        + profile.transition_score(Profile::PROFILE_BEGIN_TO_MATCH, profile_idx - 1)
                        + profile.match_score(current_residue, profile_idx)
                ),
            );
        }

        dp_matrix.set_special(
            target_idx,
            Profile::SPECIAL_J,
            log_sum!(
                dp_matrix.get_special(target_idx + 1, Profile::SPECIAL_J)
                    + profile.special_transition_score(Profile::SPECIAL_J, Profile::SPECIAL_LOOP),
                dp_matrix.get_special(target_idx, Profile::SPECIAL_B)
                    + profile.special_transition_score(Profile::SPECIAL_J, Profile::SPECIAL_MOVE)
            ),
        );

        dp_matrix.set_special(
            target_idx,
            Profile::SPECIAL_C,
            dp_matrix.get_special(target_idx + 1, Profile::SPECIAL_C)
                + profile.special_transition_score(Profile::SPECIAL_C, Profile::SPECIAL_LOOP),
        );

        dp_matrix.set_special(
            target_idx,
            Profile::SPECIAL_E,
            log_sum!(
                dp_matrix.get_special(target_idx, Profile::SPECIAL_J)
                    + profile.special_transition_score(Profile::SPECIAL_E, Profile::SPECIAL_LOOP),
                dp_matrix.get_special(target_idx, Profile::SPECIAL_C)
                    + profile.special_transition_score(Profile::SPECIAL_E, Profile::SPECIAL_MOVE)
            ),
        );

        dp_matrix.set_special(
            target_idx,
            Profile::SPECIAL_N,
            log_sum!(
                dp_matrix.get_special(target_idx + 1, Profile::SPECIAL_N)
                    + profile.special_transition_score(Profile::SPECIAL_N, Profile::SPECIAL_LOOP),
                dp_matrix.get_special(target_idx, Profile::SPECIAL_B)
                    + profile.special_transition_score(Profile::SPECIAL_N, Profile::SPECIAL_MOVE)
            ),
        );

        dp_matrix.set_match(
            target_idx,
            profile_end_on_current_row,
            dp_matrix.get_special(target_idx, Profile::SPECIAL_E),
        );

        dp_matrix.set_insert(target_idx, profile_end_on_current_row, -f32::INFINITY);

        dp_matrix.set_delete(
            target_idx,
            profile_end_on_current_row,
            dp_matrix.get_special(target_idx, Profile::SPECIAL_E),
        );

        // for profile_idx in (1..profile.length).rev() {
        for profile_idx in profile_start_on_current_row..profile_end_on_current_row {
            dp_matrix.set_match(
                target_idx,
                profile_idx,
                log_sum!(
                    dp_matrix.get_match(target_idx + 1, profile_idx + 1)
                        + profile.transition_score(Profile::PROFILE_MATCH_TO_MATCH, profile_idx)
                        + profile.match_score(current_residue, profile_idx + 1),
                    dp_matrix.get_insert(target_idx + 1, profile_idx)
                        + profile.transition_score(Profile::PROFILE_MATCH_TO_INSERT, profile_idx)
                        + profile.insert_score(current_residue, profile_idx),
                    dp_matrix.get_special(target_idx, Profile::SPECIAL_E) + end_score,
                    dp_matrix.get_delete(target_idx, profile_idx + 1)
                        + profile.transition_score(Profile::PROFILE_MATCH_TO_DELETE, profile_idx)
                ),
            );

            dp_matrix.set_insert(
                target_idx,
                profile_idx,
                log_sum!(
                    dp_matrix.get_match(target_idx + 1, profile_idx + 1)
                        + profile.transition_score(Profile::PROFILE_INSERT_TO_MATCH, profile_idx)
                        + profile.match_score(current_residue, profile_idx + 1),
                    dp_matrix.get_insert(target_idx + 1, profile_idx)
                        + profile.transition_score(Profile::PROFILE_INSERT_TO_INSERT, profile_idx)
                        + profile.insert_score(current_residue, profile_idx)
                ),
            );

            dp_matrix.set_delete(
                target_idx,
                profile_idx,
                log_sum!(
                    dp_matrix.get_match(target_idx + 1, profile_idx + 1)
                        + profile.transition_score(Profile::PROFILE_DELETE_TO_MATCH, profile_idx)
                        + profile.match_score(current_residue, profile_idx + 1),
                    dp_matrix.get_delete(target_idx, profile_idx + 1)
                        + profile.transition_score(Profile::PROFILE_DELETE_TO_DELETE, profile_idx),
                    dp_matrix.get_special(target_idx, Profile::SPECIAL_E) + end_score
                ),
            );
        }
    }

    let first_target_character = target.digital_bytes[params.target_start] as usize;

    let profile_start_in_first_row = params.left_row_bounds[params.target_start];
    let profile_end_in_first_row = params.right_row_bounds[params.target_start];

    dp_matrix.set_special(
        params.target_start - 1,
        Profile::SPECIAL_B,
        dp_matrix.get_match(params.target_start, profile_start_in_first_row)
            + profile.transition_score(Profile::PROFILE_BEGIN_TO_MATCH, 0)
            + profile.match_score(first_target_character, 1),
    );

    // for profile_idx in 2..=profile.length {
    for profile_idx in (profile_start_in_first_row + 1)..=profile_end_in_first_row {
        dp_matrix.set_special(
            params.target_start - 1,
            Profile::SPECIAL_B,
            log_sum!(
                dp_matrix.get_special(params.target_start - 1, Profile::SPECIAL_B),
                dp_matrix.get_match(params.target_start, profile_idx)
                    + profile.transition_score(Profile::PROFILE_BEGIN_TO_MATCH, profile_idx - 1)
                    + profile.match_score(first_target_character, profile_idx)
            ),
        );
    }

    dp_matrix.set_special(params.target_start - 1, Profile::SPECIAL_J, -f32::INFINITY);
    dp_matrix.set_special(params.target_start - 1, Profile::SPECIAL_C, -f32::INFINITY);
    dp_matrix.set_special(params.target_start - 1, Profile::SPECIAL_E, -f32::INFINITY);
    dp_matrix.set_special(
        params.target_start - 1,
        Profile::SPECIAL_N,
        log_sum!(
            dp_matrix.get_special(params.target_start, Profile::SPECIAL_N)
                + profile.special_transition_score(Profile::SPECIAL_N, Profile::SPECIAL_LOOP),
            dp_matrix.get_special(params.target_start - 1, Profile::SPECIAL_B)
                + profile.special_transition_score(Profile::SPECIAL_N, Profile::SPECIAL_MOVE)
        ),
    );
    for profile_idx in (profile_start_in_first_row..=profile_end_in_first_row).rev() {
        dp_matrix.set_match(params.target_start - 1, profile_idx, -f32::INFINITY);
        dp_matrix.set_insert(params.target_start - 1, profile_idx, -f32::INFINITY);
        dp_matrix.set_delete(params.target_start - 1, profile_idx, -f32::INFINITY);
    }
}
