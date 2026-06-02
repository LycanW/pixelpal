/// Build the base/reference character prompt.
/// The base image becomes the canonical visual anchor for all frames.
pub fn build_base_prompt(description: &str) -> String {
  format!(
    "A pixel-art game sprite of {}. \
Compact chibi proportions, chunky readable silhouette, thick dark 1-2px outline, \
visible stepped pixel edges, limited 16-color palette, flat cel shading, \
simple expressive face, tiny limbs. \
Standing front-facing, neutral pose. Centered, full body visible. \
Transparent background. No effects, shadows, or detached elements.",
    description
  )
}

/// Build a frame prompt with codex-pet-style Identity Lock.
/// Every frame must be recognizably the SAME individual pet.
pub fn build_frame_prompt(
  base_description: &str,
  animation_name: &str,
  frame_index: u32,
  total_frames: u32,
  pose_description: &str,
) -> String {
  format!(
    "Identity lock: generate the EXACT SAME pixel-art character from the canonical reference. \
Do NOT redesign the character. Only change the specific pose/action below. \
Preserve the exact head shape, face design, ear shape, markings, color palette, outline weight, body proportions, and overall silhouette. \
Keep every frame recognizably the same individual pet, not a related variant. \
\
Frame {}/{} of the '{}' animation state: {}. \
Canonical character: {}. \
\
Style rules: \
- Transparent background, centered, full body visible. \
- Clean crisp pixel edges, pixel-art game-asset style. \
- No detached effects: no floating stars, sparkles, dust, smoke, speed lines, motion arcs, blur, smears, halos, glows, auras. \
- No shadows: no cast shadows, contact shadows, drop shadows, floor patches, landing marks. \
- No text, labels, frame numbers, grids, guide marks, speech bubbles, scenery, or checkerboard transparency. \
- No detached outline bits, stray pixels, or cropped body parts.",
    frame_index + 1,
    total_frames,
    animation_name,
    pose_description,
    base_description,
  )
}

/// Build a prompt that generates an entire row of frames in a single request.
/// This is the codex-pet approach: all frames share the same model context,
/// so identity consistency is much better than per-frame requests.
pub fn build_row_prompt(
  base_description: &str,
  animation_name: &str,
  total_frames: u32,
  pose_descriptions: &[String],
) -> String {
  let frame_list = pose_descriptions.iter().enumerate()
    .map(|(i, p)| format!("Frame {}: {}", i + 1, p))
    .collect::<Vec<_>>()
    .join("; ");

  format!(
    "A pixel-art game sprite strip of {n} frames of the SAME character, arranged left-to-right in one horizontal row. \
Compact chibi proportions, chunky readable silhouette, thick dark 1-2px outline, \
visible stepped pixel edges, limited 16-color palette, flat cel shading. \
Identity lock: exact same head shape, face, ears, markings, color palette, outline weight, body proportions, and overall silhouette in EVERY frame. \
Only the pose/action changes between frames. \
{n} frames left-to-right: {frames}. \
Character: {desc}. State: {anim}. \
Transparent background. Equal-width slots, one complete pose per slot. No pose crosses into neighboring slots. \
No effects, shadows, text, scenery, detached elements.",
    n = total_frames,
    frames = frame_list,
    desc = base_description,
    anim = animation_name,
  )
}

/// Get pose descriptions for each animation state.
/// idle = neutral breathing/blinking loop (codex-pet style).
/// Each frame must show the SAME standing pose with only subtle expression changes.
pub fn get_pose_sequence(animation_name: &str, total_frames: u32) -> Vec<String> {
  let default = match animation_name {
    "idle" => vec![
      "standing neutral, eyes wide open, chest slightly expanded (breathing in)",
      "standing neutral, eyelids lowering (mid-blink), neutral breathing",
      "standing neutral, eyes fully closed (blink peak), chest slightly contracted (breathing out)",
      "standing neutral, eyelids rising (recovering from blink), neutral breathing",
    ],
    "walk" => vec![
      "left foot forward, right foot back, natural arm swing",
      "both feet neutral mid-stride, arms at sides",
      "right foot forward, left foot back, natural arm swing",
      "both feet neutral mid-stride, arms at sides",
    ],
    "run" => vec![
      "left leg forward high, right arm forward, leaning forward",
      "both feet off ground mid-air, arms pumping",
      "right leg forward high, left arm forward, leaning forward",
      "both feet off ground mid-air, arms pumping",
    ],
    "react" => vec![
      "surprised jump, arms raising up",
      "peak surprise, eyes wide, arms up",
      "settling down, arms lowering slowly",
      "return to neutral standing",
    ],
    "sleep" => vec![
      "lying down curled up, eyes closed, peaceful",
      "sleeping, slight breathing movement",
      "sleeping deeply, tiny Z motion",
      "sleeping, slight breathing movement",
    ],
    _ => vec!["neutral pose"],
  };

  (0..total_frames)
    .map(|i| default[i as usize % default.len()].to_string())
    .collect()
}
