# Matrix Bot

Matrix Bot is a simple matrix bot aimed at medium to large projects that span many chat rooms and many active repos looking for more options than the official matrix bots can provide.

## Features

- ### A jokey interjection that can correct pesky users that misspell your project name!
    
    Off by default
    
    Can prevent from running in specific rooms
    
    Has fully configurable matches (case sensitive *and* insentive) and response text

    Has 5 minute cooldown timer per room for less spam and more fun

- ### An imperial <--> metric converter for all messages containing common units!
    
    This can be disabled entirely and some units can be excluded from spaced matches if they are also a word (eg: 'i got a 500 in response')

    Eases idle chitchat between community members

- ### A configurable search and link for issues/pulls any Github repos the supplied Github user can see!
    This can be turned off by not supplying any repos to search
    
    Searches are parsed from message text if they match 'jf#123' or 'jf #123'
    
    The left side of the # is configrable and can point to any repo
    
    Uses GraphQL to be API cost effective (REST might require 2 hits depending on returned result)

- ### A configrable general purpose linker!
    
    This can be turned off by supplying no linkable urls

    Links anything matched from a parsed message if it contains 'docs@hwa' or 'link @troubleshooting'

    Left side is configurable. All linkable urls can be triggered with all keywords

    Right side is configrable and can link to any URL

## Installation and configuration

Currently there is no 
