#!/bin/bash

# Demo: Multi-project workspace mode

set -e

echo "🌐 gidterm Multi-Project Workspace Demo"
echo "========================================"
echo ""
echo "This demo shows gidterm managing multiple projects simultaneously."
echo ""
echo "Workspace structure:"
echo "  test-workspace/"
echo "    ├── project-a/  (Backend API)"
echo "    │   └── .gid/graph.yml"
echo "    └── project-b/  (Frontend UI)"
echo "        └── .gid/graph.yml"
echo ""
echo "Starting gidterm in workspace mode..."
echo "(Press 'q' to quit when done)"
echo ""
sleep 2

cd test-workspace
cargo run -- --workspace

echo ""
echo "✨ Demo complete!"
