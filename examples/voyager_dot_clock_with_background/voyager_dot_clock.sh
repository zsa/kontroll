#!/bin/bash

kontroll set-rgb-all --color "#000000"

hour_fill=(   "#ff00ff"   "#ff00ff"   "#0000ff"   "#0000ff"   "#ff00ff"
    "#ff00ff"   "#0000ff"   "#ff00ff"   "#ff00ff"   "#0000ff"   "#0000ff"
    "#ff00ff"   "#ff00ff"   "#ff00ff"   "#0000ff"   "#0000ff"   "#ff00ff"
    "#ff00ff"   "#0000ff"   "#ff00ff"   "#ff00ff"   "#0000ff"   "#0000ff"
    "#ff00ff") 
hour_bg=(   "#060006"   "#060006"   "#000000"   "#000000"   "#060006"
    "#060006"   "#000000"   "#060006"   "#060006"   "#000000"   "#000000"
    "#060006"   "#060006"   "#060006"   "#000000"   "#000000"   "#060006"
    "#060006"   "#000000"   "#060006"   "#060006"   "#000000"   "#000000"
    "#060006") 
hour_ids=( 0 6 1 7 2 8 3 9 4 10 5 11 26 32 27 33 28 34 29 35 30 36 31 37)

minute5_fill=(   "#ff00ff"   "#0000ff"   "#ff00ff"   "#ff00ff"   "#0000ff"
    "#ff00ff"   "#0000ff"   "#ff00ff"   "#0000ff"   "#ff00ff"   "#ff00ff"
    "#ff00ff") 
minute5_bg=(   "#060006"   "#000000"   "#060006"   "#060006"   "#000000"
    "#060006"   "#000000"   "#060006"   "#000000"   "#060006"   "#060006"
    "#060006") 

minute5_ids=(12 13 14 15 16 17 38 39 40 41 42 43)


second5_fill=(   "#0000ff"   "#ff00ff"   "#0000ff"   "#0000ff"   "#ff00ff"
    "#0000ff"   "#0000ff"   "#ff00ff"   "#0000ff"   "#ff00ff"   "#0000ff"
    "#ff00ff") 
second5_bg=(   "#000000"   "#060006"   "#000000"   "#000000"   "#060006"
    "#000000"   "#000000"   "#060006"   "#000000"   "#060006"   "#000000"
    "#060006") 
second5_ids=(18 19 20 21 22 23 44 45 46 47 48 49)

minute4_fill=(   "#0000ff"   "#0000ff"   "#0000ff"   "#0000ff")
minute4_bg=(   "#000000"   "#000000"   "#000000"   "#000000")
minute4_ids=(24 25 50 51)

for ((i=0; i<24; i++)); do
  kontroll set-rgb --led ${hour_ids[$(($i))]} -c "${hour_bg[$i]}"
done

for ((i=0; i<12; i++)); do
  kontroll set-rgb --led ${minute5_ids[$(($i))]} -c "${minute5_bg[$(($i))]}"
done

for ((i=0; i<12; i++)); do
  kontroll set-rgb --led ${second5_ids[$(($i))]} -c "${second5_bg[$(($i))]}"
done
for ((i=0; i<4; i++)); do
  kontroll set-rgb --led ${minute4_ids[$(($i))]} -c "${minute4_bg[$(($i))]}"
done


# speed
# dt=0
# s=$((10#$(date -d "@$dt" "+%S")))
# m=$((10#$(date -d "@$dt" "+%M")))
# h=$((10#$(date -d "@$dt" "+%H")))

s=$((10#$(date "+%S")))
m=$((10#$(date "+%M")))
h=$((10#$(date "+%H")))

for ((i=1; i<=h; i++)); do
  kontroll set-rgb --led ${hour_ids[$(($i-1))]} -c "${hour_fill[$i-1]}"
done

for ((i=1; i<=m/5; i++)); do
  kontroll set-rgb --led ${minute5_ids[$(($i-1))]} -c "${minute5_fill[$i-1]}"
done

for ((i=1; i<=s/5; i++)); do
  kontroll set-rgb --led ${second5_ids[$(($i-1))]} -c "${second5_fill[$i-1]}"
done

for ((i=0; i<m%5; i++)); do
  kontroll set-rgb --led ${minute4_ids[$i]} -c "${minute4_fill[$i]}"
done

l_h=-1
l_m=-1
l_s=-1
l_m4=-1

while true; do
  # speed
  # s=$((10#$(date -d "@$dt" "+%S")))
  # m=$((10#$(date -d "@$dt" "+%M")))
  # h=$((10#$(date -d "@$dt" "+%H")))

  s=$((10#$(date "+%S")))
  m=$((10#$(date "+%M")))
  h=$((10#$(date "+%H")))

  if [ "$s" != "$l_s" ]; then
    echo "$h:$m:$s"
  fi

  if [ "$h" != "$l_h" ]; then
    if [ "#$((h))" -eq 0 ]; then 
      for ((i=0; i<24; i++)); do
        kontroll set-rgb --led ${hour_ids[$(($i))]} -c "${hour_bg[$i]}"
      done
    else 
      kontroll set-rgb --led ${hour_ids[$(($h - 1))]} -c "${hour_fill[$(($h - 1))]}"
    fi
  fi

  if [ "$m" != "$l_m" ]; then
    if [ "$((m/5))" -eq 0 ]; then 
      for ((i=0; i<12; i++)); do
        kontroll set-rgb --led ${minute5_ids[$(($i))]} -c "${minute5_bg[$(($i))]}"
      done
    else
      kontroll set-rgb --led ${minute5_ids[$(($m/5 - 1))]} -c "${minute5_fill[$(($m/5 - 1))]}"
    fi

    if [ "$((m%5))" -eq 0 ]; then 
      for ((i=0; i<4; i++)); do
        kontroll set-rgb --led ${minute4_ids[$(($i))]} -c "${minute4_bg[$(($i))]}"
      done
    else
      kontroll set-rgb --led ${minute4_ids[$(($m%5 -1))]} -c "${minute4_fill[$(($m%5 -1))]}"
    fi

  fi

  if [ "$s" != "$l_s" ]; then
    if [ "$((s/5))" -eq 0 ]; then 
      for ((i=0; i<12; i++)); do
        kontroll set-rgb --led ${second5_ids[$(($i))]} -c "${second5_bg[$(($i))]}"
      done
    else
      kontroll set-rgb --led ${second5_ids[$(($s/5 - 1))]} -c "${second5_fill[$(($s/5 - 1))]}"
    fi
  fi

  l_h=$h
  l_m=$m
  l_s=$s

  # speed
  # you can change the speed by changing this 5 or by not commenting sleep 0.1
  # dt=$(($dt+5))

  # comment for speed
  sleep 0.1
  
done
