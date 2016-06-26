# pi-b-q

Project to programatically interact with a Thermoworks BlueTherm thermomoter and provide a web interface that displays temperature over time from a Raspberry Pi

## Installation

1. Clone this repo onto a build machine (anything that'll run Docker)
1. Also clone this repo onto the Raspberry Pi
1. Run docker-compose to cross compile the binaries
1. `scp` the `web` and `harvester` binaries to the dist folder in the project tree on the Pi (binaries will be in `target/armv7-unknown-linux-gnueabihf/release`)
1. Run the install script in the root of the repo
1. Update /etc/default/pibq to reflect your BT config


pi-b-q is released under the MIT License.

Copyright (C) 2016 Dan Elbert

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
