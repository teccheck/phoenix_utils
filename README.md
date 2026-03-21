# Phoenix Utils 📟

This is a little project to interface with the DE10A pager hardware by [Swissphone](https://www.swissphone.com/), which is used in the BOSS 915 and BOSS 935 pagers. The command format should also work for other Swissphone pagers, as it's used by [PSWplus](https://www.swissphone.com/software-updates/), but I didn't test that.

This is in very early stages of development. Docs don't exist, and the code quality is lacking. There are also a lot of missing features. However, I appreciate stars and especially help. In any case, I hope you like it.

If you do want to contribute, I recommend [de4dotEx](https://github.com/GDATAAdvancedAnalytics/de4dotEx) and [dnSpyEx](https://github.com/dnSpyEx/dnSpy) for deobfuscation and decompilation.

## Disclaimer

This project is for educational purposes only. Use at your own risk. This is not endorsed by Swissphone, and I'm not connected to them or their partners in any way. I did not use any illegally obtained resources during the development of this code.

## Project structure

You'll find two main modules in here: phoenix and cli. Cli, as the name implies contains all functionality related to input and output from and to the user. Phoenix is much more interesting, as it contains all business logic and communication protocols. The name is used by Swion for their library as well, and I like it. Here you'll find submodules for commands, tasks (more complex stuff using multiple commands), protocols, types, and a string encoding. I'd say, the code is fairly understandable even if you're not used to Rust.

## TODO

- [x] Dynamic buffer size in SCI Frame protocol and raw
- [x] Error handling in commands via SwionResult and SwionError
- [ ] Move utilities into utils crate or similar
- [ ] Move CommandType into its own crate?
- [x] Implement time set/get commands
- [x] Implement backlight and led test mode commands
- [ ] Implement message log commands
- [ ] Hierarchic CLI commands
- [x] Module hierarchy for phoenix commands
- [ ] Look for more stuff to do

## More

If you're interested in the topic, there are a handful of articles on my [website](https://teccheck.xyz/blog/the-de10a-pager).
