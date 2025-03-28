# A fully unoptimized [Space Trader](https://spacetraders.io) map viewer

# How to use
1. Download the binary from the [releases page](https://github.com/carne8/space-trader/releases)
2. Create an agent on [the space traders website](https://my.spacetraders.io/agents)
3. Generate and copy the agent token
4. Run the following command in your terminal:
```bash
./space-trader.exe --token=<your_token>
```

If this is your first time running the program, it will download the [systems](https://spacetraders.io/game-concepts/systems-waypoints) and store them in a local file. This can take some time.  

One the systems are downloaded, you can omit the `--token` flag in future runs.

# Refresh the systems
If you want to re-download the systems, you can use the `--download-systems` flag:
```bash
./space-trader.exe --download-systems --token=<your_token>
```

Alternatively, you can delete the `systems.json` file in the same directory as the executable. The program will automatically download the systems again.
