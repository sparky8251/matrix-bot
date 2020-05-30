**Github ratelimiting is punishing. Implement proper handler.**
    Ratelimiting has 1hr cooldown that is reset with a *single*
    query during the period in which you have no queries left.

    Will want to store last query cost, remaining points from last query,
    and last query time and use this to determine if its safe to make another query.
    
    Make sure that there is enough points left to do next query before attempting.

    If is a rate limit in effect, have bot reply with UTC datetime that searches can resume.

**Switch to Diesel powered SQLite storage backend**
    Storage complexity is growing and could use the extra flexibility.

    Investigate use of barrel for schema management with code as the Diesel
    cli is an excessive requirement for a small bot.

    Use this more complex backend to enable a queue of unique IDs for messages in case
    a retry is required

**Look at slog as replacement for log**
    slog_term will be the first pass, but look at possibility of built in syslog shipping

**Current logging story is a problem**
    Add more logging for admins that isnt debug/trace level

    Wxamples would be logging of all triggered events, users that trigger them, 
    room they were triggered in, data they were triggered with, data they responded with,
    and so on.
    
    Allow admin to enable/disable logging of username and rooms that trigger events

**Look at having the bot respond on relevant errors with proper information**
    If the bot is unable to perform an action like github search, it should reply
    with relevant information for users.

    If its rate limited for github, reply with a UTC datetime of when it can next search.
    If its unauthorized for github, reply with a message stating that.
    If its unable to parse a number to a float, look at replying with an error message.
    (Must investigate if this will be a problem for false hits. Likely want to provide dummy number
    for conversion so I can see if the unit works and it was just a bad quantity)

**Handle various float cases**
    I should be able to detect the inf float case and fail conversion on such a large number
    I want to handle formatting of numbers with , and . as separators such as `5.000,00` or 
    `5,000.00` gracefully. Currently these fail to convert to float.

**Implement help command**
    Would prefer help command limited to specific rooms OR to invite to private rooms.
    If limited to specific room, maybe reply with message that it runs in a specific room only?
    Invites don't work for IRC so invites to the help room might be a problem.

    !help should dynamically generate replies for commands such as gh searching and docs linking
    so users can determine what can run and how to use it.

**Work on improving test coverage**
    Use more extensive unit testing to cover more cases and sure they remain functioning.
    Look into doc testing to make it much easier to contribute to this bot in the future.