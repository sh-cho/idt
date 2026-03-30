Output demo.gif

Require idt

Set Shell "bash"
Set FontSize 18
Set Width 800
Set Height 540
Set Padding 20
Set TypingSpeed 50ms

# -- gen: uuid --
Sleep 500ms
Type "idt gen uuid"
Sleep 300ms
Enter

Sleep 500ms
Type "idt gen uuidv7"
Sleep 300ms
Enter
Sleep 1.5s

# -- gen: uuidv7 batch --
Type "idt gen uuidv4 -n 10"
Sleep 300ms
Enter
Sleep 2s

# -- gen: snowflake with preset --
Type "idt gen snowflake --preset discord"
Sleep 300ms
Enter
Sleep 2s

# clear after gen section
Hide
Type "clear"
Enter
Sleep 300ms
Show

# -- inspect: isbn13
Type "idt inspect 9788966264155"
Sleep 300ms
Enter
Sleep 3s

# clear after gen section
Hide
Type "clear"
Enter
Sleep 300ms
Show

# -- pipe: gen + inspect --
Type "idt gen uuidv7 | idt inspect"
Sleep 300ms
Enter
Sleep 5s

