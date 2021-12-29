
import { set_game_name, set_loading, set_instance } from '../index'

var game_name, game_file;

document.getElementById("addbutton").onclick = function () {
    const loc = window.location;
    const query = `?game=${game_file}&name=${game_name}`;
    navigator.clipboard.writeText(loc.origin + loc.pathname + encodeURI(query)).then(() => {
        console.log("Success");
    }, () => {
        console.log("Failed");
    });
}

async function on_load() {
    const options = document.getElementsByClassName("options");
    const urlVars = new URLSearchParams(window.location.search);

    if (urlVars.get("game") && urlVars.get("name")) {
        console.log(urlVars.get("game") + ' ' + urlVars.get("name"))
        handle(urlVars.get("game"), urlVars.get("name"))
    } else if (options[0]) {
        const options_div = <HTMLDivElement>options[0];
        if (options_div.children[0]) {
            options_div.children[0].dispatchEvent(new Event('click'));
        }
    }
}

window.addEventListener("load", on_load, false);

export function handle(location: string, name: string) {
    game_file = location;
    game_name = name;

    set_loading(true);

    fetch(location)
        .then((r) => r.text())
        .then((response) => {
            set_instance(response);
            set_game_name(name);
        }).catch(console.error);
}

on_load();
