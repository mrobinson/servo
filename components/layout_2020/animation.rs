/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use crate::opaque_node::OpaqueNodeMethods;
use crate::FragmentTreeRoot;
use fxhash::{FxHashMap, FxHashSet};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::{
    AnimationState, ConstellationControlMsg, LayoutMsg as ConstellationMsg, UntrustedNodeAddress,
};
use style::animation::{Animation, ElementAnimationState};
use style::dom::OpaqueNode;

/// Processes any new animations that were discovered after style recalculation and
/// remove animations for any disconnected nodes. Send messages that trigger events
/// for any events that changed state.
pub fn do_post_style_animations_update(
    constellation_channel: &IpcSender<ConstellationMsg>,
    script_channel: &IpcSender<ConstellationControlMsg>,
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationState>,
    pipeline_id: PipelineId,
    now: f64,
    out: Option<&mut Vec<UntrustedNodeAddress>>,
    root: &FragmentTreeRoot,
) {
    let had_running_animations = animation_states
        .values()
        .any(|state| !state.running_animations.is_empty());

    cancel_animations_for_disconnected_nodes(animation_states, root);
    collect_newly_animating_nodes(animation_states, out);

    let send_event = |animation: &Animation, event_type, elapsed_time| {
        let (node, property_or_animation_name) = match *animation {
            Animation::Transition(node, _, ref property_animation) => {
                (node, property_animation.property_name().into())
            },
            Animation::Keyframes(node, _, ref name, _) => (node, name.to_string()),
        };

        script_channel
            .send(ConstellationControlMsg::TransitionOrAnimationEvent {
                pipeline_id,
                event_type,
                node: node.to_untrusted_node_address(),
                property_or_animation_name,
                elapsed_time,
            })
            .unwrap()
    };

    for animation_state in animation_states.values_mut() {
        animation_state.process_post_style(now, send_event);
    }

    // Remove empty states from our collection of states in order to free
    // up space as soon as we are no longer tracking any animations for
    // a node.
    animation_states.retain(|_, state| !state.is_empty());

    let have_running_animations = animation_states
        .values()
        .any(|state| !state.running_animations.is_empty());
    let present = match (had_running_animations, have_running_animations) {
        (true, false) => AnimationState::NoAnimationsPresent,
        (false, true) => AnimationState::AnimationsPresent,
        _ => return,
    };
    constellation_channel
        .send(ConstellationMsg::ChangeRunningAnimationsState(
            pipeline_id,
            present,
        ))
        .unwrap();
}

/// Collect newly animating nodes, which is used by the script process during
/// forced, synchronous reflows to root DOM nodes for the duration of their
/// animations or transitions.
pub fn collect_newly_animating_nodes(
    animation_states: &FxHashMap<OpaqueNode, ElementAnimationState>,
    mut out: Option<&mut Vec<UntrustedNodeAddress>>,
) {
    // This extends the output vector with an iterator that contains a copy of the node
    // address for every new animation. This is a bit goofy, but the script thread
    // currently stores a rooted node for every property that is transitioning.
    if let Some(ref mut out) = out {
        out.extend(animation_states.iter().flat_map(|(node, state)| {
            std::iter::repeat(node.to_untrusted_node_address()).take(state.new_animations.len())
        }));
    }
}

/// Cancel animations for any nodes which have been removed from the DOM or are display:none.
/// We detect this by looking for nodes that are used in the flow tree.
/// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
/// This also doesn't yet handles nodes that have been reparented.
pub fn cancel_animations_for_disconnected_nodes(
    animation_states: &mut FxHashMap<OpaqueNode, ElementAnimationState>,
    root: &FragmentTreeRoot,
) {
    // Assume all nodes have been removed until proven otherwise.
    let mut invalid_nodes: FxHashSet<OpaqueNode> = animation_states.keys().cloned().collect();
    root.find(|fragment, _| -> Option<()> {
        if let Some(tag) = fragment.tag().as_ref() {
            invalid_nodes.remove(tag);
        }
        None
    });

    // Cancel animations for any nodes that are no longer in the flow tree.
    for node in &invalid_nodes {
        if let Some(state) = animation_states.get_mut(node) {
            state.cancel_all_animations();
        }
    }
}
