pub fn build_base_prompt(description: &str) -> String {
  format!(
    "A pixel-art character standing front-facing: {}. Transparent background. Centered, full body visible. Clean crisp pixel edges. Pixel-art game-asset style.",
    description
  )
}

pub fn build_frame_prompt(base_description: &str, animation_name: &str, frame_index: u32, total_frames: u32, pose_description: &str) -> String {
  format!(
    "Same pixel-art character as reference: {}. {} pose, frame {}/{}: {}. Centered, full body visible. Transparent background. Clean crisp pixel edges. Pixel-art game-asset style.",
    base_description,
    animation_name,
    frame_index + 1,
    total_frames,
    pose_description
  )
}

pub fn get_pose_sequence(animation_name: &str, total_frames: u32) -> Vec<String> {
  let default = match animation_name {
    "idle" => vec![
      "standing neutral, eyes open",
      "standing neutral, eyes half closed",
      "standing neutral, eyes fully closed",
      "standing neutral, eyes half closed again",
    ],
    "walk" => vec![
      "left foot forward, right foot back",
      "both feet neutral",
      "right foot forward, left foot back",
      "both feet neutral",
    ],
    "run" => vec![
      "left leg forward high, right arm forward",
      "both feet off ground mid-air",
      "right leg forward high, left arm forward",
      "both feet off ground mid-air",
    ],
    "react" => vec![
      "surprised jump, arms up",
      "peak surprise, eyes wide",
      "settling down, arms lowering",
      "return to neutral",
    ],
    "sleep" => vec![
      "lying down, eyes closed",
      "sleeping peacefully",
      "slight breathing movement",
      "sleeping peacefully",
    ],
    _ => vec!["neutral pose"],
  };

  (0..total_frames)
    .map(|i| default[i as usize % default.len()].to_string())
    .collect()
}
