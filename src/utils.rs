use crate::Object;

pub fn pack_state(objects: &[Object]) -> Vec<f32> {
    let mut state = Vec::with_capacity(objects.len() * 4);
    for obj in objects {
        state.push(obj.position.x);
        state.push(obj.position.y);
        state.push(obj.velocity.x);
        state.push(obj.velocity.y);
    }
    state
}

pub fn unpack_state(state: &[f32], objects: &mut [Object]) {
    for (i, obj) in objects.iter_mut().enumerate() {
        let base = i * 4;
        obj.position.x = state[base];
        obj.position.y = state[base + 1];
        obj.velocity.x = state[base + 2];
        obj.velocity.y = state[base + 3];
    }
}
