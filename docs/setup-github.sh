#!/bin/bash
# Run this after authenticating with: gh auth login
#
# Sets up the GitHub repository metadata for MaximilianCF/fleet-canvas.
# NOTE: Rust crate names remain rmf_site_* for upstream compatibility.

REPO="MaximilianCF/fleet-canvas"

echo "Setting repository description..."
gh repo edit "$REPO" \
  --description "Fleet Canvas — visual editor for Open-RMF robot fleet deployment sites. Bevy + egui, Linux desktop."

echo "Setting repository topics..."
gh api -X PUT "repos/$REPO/topics" \
  -f '{"names":["robotics","rmf","bevy","rust","editor","gazebo","sdf","ros2","egui","linux"]}' \
  --input - <<EOF
{"names":["robotics","rmf","bevy","rust","editor","gazebo","sdf","ros2","egui","linux"]}
EOF

echo "Setting homepage URL..."
gh repo edit "$REPO" --homepage "https://github.com/$REPO/releases"

echo ""
echo "Done! Next steps:"
echo "1. Take screenshots of the editor and save them to docs/screenshots/"
echo "   - editor-overview.png: Main editor with a site loaded (nav graph visible)"
echo "   - graph-view.png: Graph View mode (F4) with color-coded lanes"
echo "   - sdf-export.png: SDF Export dialog"
echo "2. Upload a social preview image (1280x640px) at:"
echo "   https://github.com/$REPO/settings"
echo "   Use docs/social-preview.png if available."
echo "3. Commit screenshots and push."
