use std::collections::VecDeque;

use macroquad::{
    color::{GREEN, RED},
    math::Vec2,
};

use crate::{
    phys::{PredictionDirection, Predictor},
    render::{DPAllocations, draw_prediction},
    state::State,
};

pub fn draw_prediction_ui(state: &mut State) {
    if state.prediction_dirty {
        if state.predict_future {
            let max_steps = (state.fw_predict_d_epoch / state.fixed_dt).round() as usize;
            let objects = state.objects.read().unwrap();
            state.future_predictor = Some(Predictor {
                objects: objects.clone(),
                objects_terminal: objects.clone(),
                path: VecDeque::with_capacity(max_steps * objects.len()),
                steps_completed: 0,
                max_steps,
                direction: PredictionDirection::Future,
            });

            state.future_predictor.as_mut().unwrap().simulate_steps(
                &mut state.y4_integrator,
                state.fixed_dt,
                objects.len(),
            );

            state.fw_pred_d_allocs = Some(DPAllocations {
                // The color given here is actually completely irrelevent
                colors: vec![GREEN; objects.len() * max_steps].into_boxed_slice(),
                path_data: vec![Vec2::ZERO; max_steps].into_boxed_slice(),
            });
        } else {
            state.future_predictor = None;
            state.fw_pred_d_allocs = None;
        }
        if state.predict_past {
            let max_steps = (state.bw_predict_d_epoch / state.fixed_dt).round() as usize;
            let objects = state.objects.read().unwrap();
            state.past_predictor = Some(Predictor {
                objects: objects.clone(),
                objects_terminal: objects.clone(),
                path: VecDeque::with_capacity(max_steps * objects.len()),
                steps_completed: 0,
                max_steps,
                direction: PredictionDirection::Past,
            });

            state.past_predictor.as_mut().unwrap().simulate_steps(
                &mut state.y4_integrator,
                -state.fixed_dt,
                objects.len(),
            ); // This simulation leaves the terminal, ready to be rolled
            //
            state.bw_pred_d_allocs = Some(DPAllocations {
                colors: vec![RED; objects.len() * max_steps].into_boxed_slice(),
                path_data: vec![Vec2::ZERO; max_steps].into_boxed_slice(),
            });
        } else {
            state.past_predictor = None;
            state.bw_pred_d_allocs = None;
        }
        state.prediction_dirty = false;
    }

    if let Some(pred) = &state.future_predictor {
        let allocs = state.fw_pred_d_allocs.as_mut().unwrap();
        draw_prediction(
            allocs,
            &pred.path,
            pred.objects.len(),
            pred.max_steps,
            GREEN,
            state.fw_orbit_line_fade,
        );
    }
    if let Some(pred) = &state.past_predictor {
        let allocs = state.bw_pred_d_allocs.as_mut().unwrap();
        draw_prediction(
            allocs,
            &pred.path,
            pred.objects.len(),
            pred.max_steps,
            RED,
            state.bw_orbit_line_fade,
        );
    }
}
