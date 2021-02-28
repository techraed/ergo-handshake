# TODO
## Schedule
1. refactoring
2. review and discussions on some minor todos
3. resolve postponed stuff
## Refactoring
1. Check re-exports and visibilities
2. Handshake message: 2.1) move HsSpecReader/Writer closer to messages (either spec dir or, likely, messages/specs),
 default vlq reader/writer leave in utils, 2.2) HsSpecReader/Writer must have their own error types, 2.3) clean-up these modules.
3. hs function must return it's own error types. Define it in lib/
## Postponed
1. Proper tests for HS with timestamp   
2. Optimize VLQ (de)encoder