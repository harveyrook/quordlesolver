# quordlesolver
quordle solver

Execute with...
  **cargo run --release play**
  
quorlesolver will respond with a guess. After you enter the guess in quordle, enter the clue you got back.

To enter the clue, use " " for missed letters, "Y" for yellow letters and "G" for green. The clue you enter should look
something like "  Y  " or " G  Y"

There are a few other options that just calculate stats for the words

Quordlesovler uses [Shanon Entropy](https://en.wikipedia.org/wiki/Entropy_(information_theory)) to rank the possible words. As such, "SOARE" is the first guess. A few other ways of ranking the words are built in, but commented out. From experience, Shanon Entropy is slightly better than the others.

My next goal is to evaulate that rigorusly.

