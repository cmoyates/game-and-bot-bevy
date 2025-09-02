# Game and Bot: Bevy

I'm remaking my old project, "Game and Bot", which was originally a Unity game exported as a WebGL build to run in the browser. The link is still live on GitHub Pages, so it should remain accessible for a while. I'm proud of the logic behind it, especially the large-scale enemy pathfinding and procedural dungeon level generation, which are aspects I really like.

However, much of the surrounding code has aged poorly, and I believe I can improve it significantly. This time, I'll remake it using Rust and the Bevy game engine. The game is pretty simple, so it doesn't require a complex engine like Unity or Godot. With Bevy, I can also export it as WASM to host it online again. That's the plan moving forward.

In the old Unity version of my game and bot project, I relied on the built-in 2D physics engine to push the rooms apart. However, after reviewing the original guides, I found they recommend using a separation steering behavior instead. Since there isn't a good small physics engine build available right now, this seems like the practical approach.

I had the AI implement basic separation behavior for the rooms. Previously, this was handled solely by the 2D physics engine in Unity. I asked it to implement the separation behavior as if it was working with voids.

Additionally, I requested a message to be logged to the console when all movement had stopped. Initially, I tried to detect this based on whether all movement ceased, but that was unreliable since technically there is no movement at the very beginning.

Instead, I asked it to print the message to the console when there were no more overlaps. This condition reliably indicates that there is no more movement.

## Resources

- <https://www.reddit.com/r/gamedev/comments/1dlwc4/procedural_dungeon_generation_algorithm_explained/>
- <https://www.gamedeveloper.com/programming/procedural-dungeon-generation-algorithm>
- <https://www.red3d.com/cwr/steer/gdc99/>
- <https://www.youtube.com/watch?v=fv-wlo8yVhk>
- <https://www.youtube.com/watch?v=mhjuuHl6qHM>
- <https://www.youtube.com/watch?v=p8d8TKo59LU>
