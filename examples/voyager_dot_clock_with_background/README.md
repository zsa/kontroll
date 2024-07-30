# Digital Dot clock with background
This script is made for the voyager keyboard. It displays the current time in a
way where you have to count the active (bright LEDs) keys to know what time it
is.

![example](./example.jpg)

The time in the picture is 01:42:25 +- 5s.

## Time
In the 1st row every key represents an odd hour (1,3,5,..). The 2nd row
represents even hours (24 doesn't light up. It's treated as 0). 

The 3rd row and the thumb keys represent minutes. Every key on the 3rd row
represents 5 minutes and the thumb keys represent the remainder.

The 4th row represents seconds where each key means 5 seconds

## Color
The fill and background color of each key can be configured. You can make something that resembles letters.

## Test
You can speed up the time by uncommenting every piece of code that says "speed".
