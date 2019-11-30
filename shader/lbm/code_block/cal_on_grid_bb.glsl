// on-grid bounce back 
if (isBounceBackCell(material)) {
  // find lattice that direction quantities flowed in
  // bounce back the direction quantities to that lattice
  for (int i = 0; i < 9; i++) {
    // lattice coords that will bounce back to
    ivec2 streaming_uv = uv + ivec2(e(REVERSED_DERECTION[i]));
    if (streaming_uv.x >= 0 && streaming_uv.x < int(lattice_num.x) &&
        streaming_uv.y >= 0 && streaming_uv.y < int(lattice_num.y)) {
      streamingCells[latticeIndex(streaming_uv) + REVERSED_DERECTION[i]] =
          collidingCells[latticeIndex(streaming_uv) + i];
    }
  }
}