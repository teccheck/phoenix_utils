# DE10A Utils

This is a little project to interface with the DE10A pager hardware by [Swissphone](https://www.swissphone.com/), which is used in the BOSS 915 and BOSS 935 pagers. The command format should also work for other swissphone pagers, as it's used by [PSWplus](https://www.swissphone.com/software-updates/), but I didn't test that.

This is in very early stages of development. Docs don't exist, and the code quality is lacking. There are also a lot of missing features. However, I appreciate stars and especially help. In any case, I hope you like it.

## TODO

- [ ] Error handling in commands via SwionResult and SwionError
- [ ] Move utilities into utils crate or similar
- [ ] Move CommandType into its own crate?
- [ ] Implement time set/get commands
- [ ] Implement backlight and led test mode commands
- [ ] Implement message log commands
- [ ] Look for more stuff to do
