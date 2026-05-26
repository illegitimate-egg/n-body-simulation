use crate::Object;

pub fn pack_state(objects: &[Object], state: &mut Vec<f32>) {
    state.resize(objects.len() * 4, 0.0);
    for (i, obj) in objects.iter().enumerate() {
        state[i * 4] = obj.position.x;
        state[i * 4 + 1] = obj.position.y;
        state[i * 4 + 2] = obj.velocity.x;
        state[i * 4 + 3] = obj.velocity.y;
    }
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
