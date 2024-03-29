import { Game } from "planetwars-rs";
import type { Dictionary } from './webgl/util';
import type { BBox } from "./voronoi/voronoi-core";

import {
  Resizer,
  resizeCanvasToDisplaySize,
  FPSCounter,
} from "./webgl/util";
import {
  Shader,
  Uniform4f,
  Uniform3fv,
  Uniform1f,
  Uniform2f,
  ShaderFactory,
  Uniform3f,
  UniformMatrix3fv,
  UniformBool,
} from "./webgl/shader";
import { DefaultRenderable, Renderer } from "./webgl/renderer";
import { VertexBuffer, IndexBuffer } from "./webgl/buffer";
import { VertexBufferLayout, VertexArray } from "./webgl/vertexBufferLayout";
import { VoronoiBuilder } from "./voronoi/voronoi";
import * as assets from "./assets";
import { loadImage, Texture } from "./webgl/texture";
import { defaultMsdfLabelFactory, MsdfLabelFactory, Label as MsdfLabel, Align } from "./webgl/msdf_text";
import { planetAtlasJson } from "./assets";


function to_bbox(box: number[]): BBox {
  return {
    xl: box[0],
    xr: box[0] + box[2],
    yt: box[1],
    yb: box[1] + box[3],
  };
}

export function set_game_name(name: string) {
  ELEMENTS["name"].innerHTML = name;
}

export function set_loading(loading: boolean) {
  if (loading) {
    if (!ELEMENTS["main"].classList.contains("loading")) {
      ELEMENTS["main"].classList.add("loading");
    }
  } else {
    ELEMENTS["main"].classList.remove("loading");
  }
}

// this function should be called after resizes happen
function do_resize() {
  resizeCanvasToDisplaySize(CANVAS);

  if (game_instance) {
    game_instance.on_resize();
  }
}

function clamp(min: number, max: number, value: number): number {
  if (value < min) {
    return min;
  }
  if (value > max) {
    return max;
  }
  return value;
}

/*
    cyrb53 (c) 2018 bryc (github.com/bryc)
    A fast and simple hash function with decent collision resistance.
    Largely inspired by MurmurHash2/3, but with a focus on speed/simplicity.
    Public domain. Attribution appreciated.
*/
const cyrb53 = function(str, seed = 0) {
  let h1 = 0xdeadbeef ^ seed, h2 = 0x41c6ce57 ^ seed;
  for (let i = 0, ch; i < str.length; i++) {
      ch = str.charCodeAt(i);
      h1 = Math.imul(h1 ^ ch, 2654435761);
      h2 = Math.imul(h2 ^ ch, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1>>>16), 2246822507) ^ Math.imul(h2 ^ (h2>>>13), 3266489909);
  h2 = Math.imul(h2 ^ (h2>>>16), 2246822507) ^ Math.imul(h1 ^ (h1>>>13), 3266489909);
  return 4294967296 * (2097151 & h2) + (h1>>>0);
};


const ELEMENTS: any = {};
var CANVAS: any;
var RESOLUTION: any;
var GL: any;
var ms_per_turn: any;

const LAYERS = {
  vor: -1, // Background
  planet: 1,
  planet_label: 2,
  ship: 3,
  ship_label: 4,
};

const COUNTER = new FPSCounter();



export function init() {
  [
    "name",
    "turnCounter",
    "main",
    "turnSlider",
    "fileselect",
    "speed",
    "canvas",
  ].forEach((n) => (ELEMENTS[n] = document.getElementById(n)));
  
  CANVAS = ELEMENTS["canvas"];
  RESOLUTION = [CANVAS.width, CANVAS.height];
  
  ms_per_turn = parseInt(ELEMENTS["speed"].value);
  
  GL = CANVAS.getContext("webgl", { antialias: true });
  
  GL.clearColor(0, 0, 0, 1);
  GL.clear(GL.COLOR_BUFFER_BIT);
  
  GL.enable(GL.BLEND);
  GL.blendFunc(GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA);

  new ResizeObserver(do_resize).observe(ELEMENTS["canvas"]);
  
  ELEMENTS["turnSlider"].oninput = function () {
    if (game_instance) {
      game_instance.updateTurn(parseInt(ELEMENTS["turnSlider"].value));
    }
  };
  
  ELEMENTS["speed"].onchange = function () {
    ms_per_turn = parseInt(ELEMENTS["speed"].value);
  };
}

export class GameInstance {
  resizer: Resizer;
  game: Game;

  shader: Shader;
  vor_shader: Shader;
  image_shader: Shader;
  masked_image_shader: Shader;
  msdf_shader: Shader;

  msdf_text_factory: MsdfLabelFactory;
  planet_labels: MsdfLabel[];
  ship_labels: MsdfLabel[];

  ship_ibo: IndexBuffer;
  ship_vao: VertexArray;
  ship_texture: Texture;
  // TODO: find a better way
  max_num_ships: number;

  renderer: Renderer;
  planet_count: number;
  planet_names: string[];

  vor_builder: VoronoiBuilder;

  vor_counter = 3;
  use_vor = true;
  playing = true;
  prev_time: DOMHighResTimeStamp = 0;


  turn: number = 0;
  // non-discrete part of visualizer time
  fractional_game_time: number = 0.0;

  turn_count = 0;

  constructor(
    game: Game,
    planet_atlas: Texture,
    ship_texture: Texture,
    robotoMsdfTexture: Texture,
    shaders: Dictionary<ShaderFactory>
  ) {
    this.game = game;
    const planets = game.get_planets();
    this.planet_count = planets.length / 3;

    this.planet_names = [];
    for (let i = 0; i < this.planet_count; i++) {
      this.planet_names.push(this.game.get_planet_name(i));
    }

    this.shader = shaders["normal"].create_shader(GL, {
      MAX_CIRCLES: "" + planets.length,
    });
    this.image_shader = shaders["image"].create_shader(GL);
    this.vor_shader = shaders["vor"].create_shader(GL, {
      PLANETS: "" + planets.length,
    });
    this.masked_image_shader = shaders["masked_image"].create_shader(GL);

    this.msdf_shader = shaders["msdf"].create_shader(GL);
    this.msdf_text_factory = defaultMsdfLabelFactory(GL, robotoMsdfTexture, this.msdf_shader);
    this.planet_labels = [];
    this.ship_labels = [];

    this.ship_texture = ship_texture

    this.resizer = new Resizer(CANVAS, [...game.get_viewbox()], true);
    this.renderer = new Renderer();
    this.game.update_turn(0);

    // Setup key handling
    document.addEventListener("keydown", this.handleKey.bind(this));

    // List of [(x, y, r)] for all planets
    this._create_voronoi(planets);
    this._create_planets(planets, planet_atlas);

    this.max_num_ships = 0;

    // Set slider correctly
    this.turn_count = game.turn_count();
    ELEMENTS["turnSlider"].max = this.turn_count - 1 + "";
  }

  push_state(state: string) {
      this.game.push_state(state);

      if (this.turn == this.turn_count - 1) {
        this.playing = true;
      }
      
      // Set slider correctly
      this.turn_count = this.game.turn_count();
      this.updateTurnCounters();
  }

  _create_voronoi(planets: Float32Array) {
    const planet_points = [];
    for (let i = 0; i < planets.length; i += 3) {
      planet_points.push({ x: -planets[i], y: -planets[i + 1] });
    }

    const bbox = to_bbox(this.resizer.get_viewbox());

    this.vor_builder = new VoronoiBuilder(
      GL,
      this.vor_shader,
      planet_points,
      bbox
    );
    this.renderer.addRenderable(this.vor_builder.getRenderable(), LAYERS.vor);
  }

  _create_planets(planets: Float32Array, planet_atlas: Texture) {
    for (let i = 0; i < this.planet_count; i++) {
      {
        const transform = new UniformMatrix3fv([
          1,                0,                    0,
          0,                1,                    0,
          -planets[i * 3],  -planets[i * 3 + 1],  1, // TODO: why are negations needed?
        ]);

        const gl = GL;
        const ib = new IndexBuffer(gl, [
          0, 1, 2,
          1, 2, 3
        ]);
        const vb_pos = new VertexBuffer(gl, [
          -1,  1,
           1,  1,
          -1, -1,
           1, -1
        ]);

        const textureData = planetAtlasJson[cyrb53(this.planet_names[i]) % planetAtlasJson.length];
        // apply half-pixel correction to prevent texture bleeding
        // we should address the center of each texel, not the border
        // https://gamedev.stackexchange.com/questions/46963/how-to-avoid-texture-bleeding-in-a-texture-atlas
        const x0 = (textureData.x + 0.5) / planet_atlas.getWidth();
        const x1 = (textureData.x + textureData.w - 0.5) / planet_atlas.getWidth();
        const y0 = (textureData.y + 0.5) / planet_atlas.getHeight();
        const y1 = (textureData.y + textureData.h - 0.5) / planet_atlas.getHeight();

        const vb_tex = new VertexBuffer(gl, [
          x0, y0,
          x1, y0,
          x0, y1,
          x1, y1]);
    
        const layout_pos = new VertexBufferLayout();
        // 2?
        layout_pos.push(gl.FLOAT, 2, 4, "a_position");
    
        const layout_tex = new VertexBufferLayout();
        layout_tex.push(gl.FLOAT, 2, 4, "a_texCoord");
    
        const vao = new VertexArray();
        vao.addBuffer(vb_pos, layout_pos);
        vao.addBuffer(vb_tex, layout_tex);
        
        const uniforms = {
          u_trans: transform,
          u_trans_next: transform,
        };
  
        const renderable = new DefaultRenderable(ib, vao, this.masked_image_shader, [planet_atlas], uniforms);
    
        this.renderer.addRenderable(renderable, LAYERS.planet);
    
      }

      {
        const transform = new UniformMatrix3fv([
          1,
          0,
          0,
          0,
          1,
          0,
          -planets[i * 3],
          -planets[i * 3 + 1] + 0.2 - 2*1.171875,
          1,
        ]);

        const label = this.msdf_text_factory.build(GL, transform);
        this.planet_labels.push(label);
        this.renderer.addRenderable(label.getRenderable(), LAYERS.planet_label);
      }

      {
        const transform = new UniformMatrix3fv([
          1,
          0,
          0,
          0,
          1,
          0,
          -planets[i * 3],
          -planets[i * 3 + 1] + 0.2 - 1*1.171875,
          1,
        ]);

        const label = this.msdf_text_factory.build(GL, transform);
        label.setText(GL, this.planet_names[i], Align.Middle, Align.Begin);
        this.renderer.addRenderable(label.getRenderable(), LAYERS.planet_label);
      }
    }
  }

  on_resize() {
    this.resizer = new Resizer(CANVAS, [...this.game.get_viewbox()], true);
    const bbox = to_bbox(this.resizer.get_viewbox());
    this.vor_builder.resize(GL, bbox);
  }

  _update_state() {
    this._update_planets();
    this._update_ships();
  }

  _update_planets() {
    const colours = this.game.get_planet_colors();
    const planet_ships = this.game.get_planet_ships();

    this.vor_shader.uniform(GL, "u_planet_colours", new Uniform3fv(colours));

    for (let i = 0; i < this.planet_count; i++) {
      const u = new Uniform3f(
        colours[i * 6],
        colours[i * 6 + 1],
        colours[i * 6 + 2]
      );
      this.renderer.updateUniform(
        i,
        (us) => (us["u_color"] = u),
        LAYERS.planet
      );
      const u2 = new Uniform3f(
        colours[i * 6 + 3],
        colours[i * 6 + 4],
        colours[i * 6 + 5]
      );
      this.renderer.updateUniform(
        i,
        (us) => (us["u_color_next"] = u2),
        LAYERS.planet
      );

      this.planet_labels[i].setText(
        GL,
        "" + planet_ships[i],
        Align.Middle,
        Align.Begin
      );
    }
  }

  _update_ships() {
    const ships = this.game.get_ship_locations();
    const labels = this.game.get_ship_label_locations();
    const ship_counts = this.game.get_ship_counts();
    const ship_colours = this.game.get_ship_colours();

    for (let i = this.max_num_ships; i < ship_counts.length; i++) {
      const gl = GL;
      const ib = new IndexBuffer(gl, [
        0, 1, 2,
        1, 2, 3
      ]);
      const ratio = this.ship_texture.getWidth() / this.ship_texture.getHeight();
      const vb_pos = new VertexBuffer(gl, [
        -ratio,  1,
         ratio,  1,
        -ratio, -1,
         ratio, -1
      ]);
      const vb_tex = new VertexBuffer(gl, [
        0, 0,
        1, 0,
        0, 1,
        1, 1,
      ]);
  
      const layout_pos = new VertexBufferLayout();
      layout_pos.push(gl.FLOAT, 2, 4, "a_position");
  
      const layout_tex = new VertexBufferLayout();
      layout_tex.push(gl.FLOAT, 2, 4, "a_texCoord");
  
      const vao = new VertexArray();
      vao.addBuffer(vb_pos, layout_pos);
      vao.addBuffer(vb_tex, layout_tex);

      const renderable = new DefaultRenderable(ib, vao, this.masked_image_shader, [this.ship_texture], {});
      this.renderer.addRenderable(renderable, LAYERS.ship);
      const label = this.msdf_text_factory.build(GL);

      this.ship_labels.push(label);
      this.renderer.addRenderable(label.getRenderable(), LAYERS.ship_label);
    }
    if (ship_counts.length > this.max_num_ships)
      this.max_num_ships = ship_counts.length;

    // TODO: actually remove obsolete ships
    for (let i = 0; i < this.max_num_ships; i++) {
      if (i < ship_counts.length) {
        this.ship_labels[i].setText(
          GL,
          "" + ship_counts[i],
          Align.Middle,
          Align.Middle
        );

        this.renderer.enableRenderable(i, LAYERS.ship);
        this.renderer.enableRenderable(i, LAYERS.ship_label);

        const u = new Uniform3f(
          ship_colours[i * 3],
          ship_colours[i * 3 + 1],
          ship_colours[i * 3 + 2]
        );

        const t1 = new UniformMatrix3fv(ships.slice(i * 18, i * 18 + 9));
        const t2 = new UniformMatrix3fv(ships.slice(i * 18 + 9, i * 18 + 18));

        const tl1 = new UniformMatrix3fv(labels.slice(i * 18, i * 18 + 9));
        const tl2 = new UniformMatrix3fv(labels.slice(i * 18 + 9, i * 18 + 18));

        this.renderer.updateUniform(
          i,
          (us) => {
            us["u_color"] = u;
            us["u_color_next"] = u;
            us["u_trans"] = t1;
            us["u_trans_next"] = t2;
          },
          LAYERS.ship
        );

        this.renderer.updateUniform(
          i,
          (us) => {
            us["u_trans"] = tl1;
            us["u_trans_next"] = tl2;
          },
          LAYERS.ship_label
        );
      } else {
        this.renderer.disableRenderable(i, LAYERS.ship);
        this.renderer.disableRenderable(i, LAYERS.ship_label);
      }
    }
  }

  render(timestamp: DOMHighResTimeStamp) {
    const elapsed = timestamp - this.prev_time;
    this.prev_time = timestamp;

    COUNTER.frame(timestamp);

    if (COUNTER.delta(timestamp) < 30) {
      this.vor_counter = Math.min(3, this.vor_counter + 1);
    } else {
      this.vor_counter = Math.max(-3, this.vor_counter - 1);
    }

    if (this.vor_counter < -2) {
      this.use_vor = false;
    }

    const shaders_to_update = [
      this.shader,
      this.image_shader,
      this.masked_image_shader,
      this.msdf_shader,
    ];


    // If not playing, still render with different viewbox, so that panning is still possible
    if (!this.playing) {
      shaders_to_update.forEach((shader) => {
        shader.uniform(
          GL,
          "u_viewbox",
          new Uniform4f(this.resizer.get_viewbox())
        );  
      })

      this.vor_shader.uniform(
        GL,
        "u_viewbox",
        new Uniform4f(this.resizer.get_viewbox())
      );

      this.renderer.render(GL);
      return;
    }

    this.fractional_game_time += elapsed / ms_per_turn;

    this.updateTurn(this.turn + Math.floor(this.fractional_game_time));
    this.fractional_game_time %= 1

    // TODO
    if (this.turn == this.turn_count - 1) {
      this.playing = false;
      this.fractional_game_time = 0;
    }

    // Do GL things
    GL.bindFramebuffer(GL.FRAMEBUFFER, null);
    GL.viewport(0, 0, GL.canvas.width, GL.canvas.height);
    GL.clear(GL.COLOR_BUFFER_BIT | GL.DEPTH_BUFFER_BIT);

    this.vor_shader.uniform(
      GL,
      "u_time",
      new Uniform1f(this.fractional_game_time)
    );
    this.vor_shader.uniform(
      GL,
      "u_viewbox",
      new Uniform4f(this.resizer.get_viewbox())
    );
    this.vor_shader.uniform(GL, "u_resolution", new Uniform2f(RESOLUTION));
    this.vor_shader.uniform(GL, "u_vor", new UniformBool(this.use_vor));

    shaders_to_update.forEach((shader) => {
      shader.uniform(
        GL,
        "u_time",
        new Uniform1f(this.fractional_game_time)
      );
      shader.uniform(
        GL,
        "u_mouse",
        new Uniform2f(this.resizer.get_mouse_pos())
      );
      shader.uniform(
        GL,
        "u_viewbox",
        new Uniform4f(this.resizer.get_viewbox())
      );
      shader.uniform(GL, "u_resolution", new Uniform2f(RESOLUTION));
    });

    // Render
    this.renderer.render(GL);

    COUNTER.frame_end();
  }

  updateTurn(turn: number) {
    this.turn = clamp(0, this.turn_count-1, turn);
    this.game.update_turn(this.turn);
    this._update_state();
    this.updateTurnCounters();
  }

  updateTurnCounters() {
    ELEMENTS["turnCounter"].innerHTML =
      this.turn + " / " + (this.turn_count - 1);
    ELEMENTS["turnSlider"].value = this.turn + "";
    ELEMENTS["turnSlider"].max = this.turn_count - 1 + "";
  } 

  handleKey(event: KeyboardEvent) {
    let delta = event.shiftKey ? 5 : 1;
    switch (event.code) {
      case "Space":
        this.playing = !this.playing;
        break;
      case "ArrowLeft":
        this.updateTurn(this.turn - delta);
        break;
      case "ArrowRight":
        this.updateTurn(this.turn + delta);
        break;
    }
  }
}

var game_instance: GameInstance;
var texture_images: HTMLImageElement[];
var shaders: Dictionary<ShaderFactory>;

export async function set_instance(source: string): Promise<GameInstance> {
  // TODO: this loading code is a mess. Please clean me up!
  if (!texture_images || !shaders) {
    const image_promises = [
      loadImage(assets.shipPng),
      loadImage(assets.planetAtlasPng),
      loadImage(assets.robotoMsdfPng),
    ];

    const shader_promies = [
      (async () =>
        <[string, ShaderFactory]>[
          "normal",
          await ShaderFactory.create_factory(
            assets.simpleFragmentShader,
            assets.simpleVertexShader,
          ),
        ])(),
      (async () =>
        <[string, ShaderFactory]>[
          "vor",
          await ShaderFactory.create_factory(
            assets.vorFragmentShader,
            assets.vorVertexShader,
          ),
        ])(),
      (async () =>
        <[string, ShaderFactory]>[
          "image",
          await ShaderFactory.create_factory(
            assets.imageFragmentShader,
            assets.simpleVertexShader,
          ),
        ])(),
      (async () =>
        <[string, ShaderFactory]>[
          "masked_image",
          await ShaderFactory.create_factory(
            assets.maskedImageFragmentShader,
            assets.simpleVertexShader,
          ),
        ])(),
      (async () =>
        <[string, ShaderFactory]>[
          "msdf",
          await ShaderFactory.create_factory(
            assets.msdfFragmentShader,
            assets.simpleVertexShader,
          ),
        ])(),
    ];
    let shaders_array: [string, ShaderFactory][];
    [texture_images, shaders_array] = await Promise.all([
      Promise.all(image_promises),
      Promise.all(shader_promies),
    ]);

    shaders = {};
    shaders_array.forEach(([name, fac]) => (shaders[name] = fac));
  }

  resizeCanvasToDisplaySize(CANVAS);
  const shipTexture = Texture.fromImage(GL, texture_images[0], "ship");
  const planetTexture = Texture.fromImage(GL, texture_images[1], "planetAtlas");
  const robotoMsdfTexture = Texture.fromImage(GL, texture_images[2], "robotoMsdf");


  game_instance = new GameInstance(
    Game.new(source),
    planetTexture,
    shipTexture,
    robotoMsdfTexture,
    shaders
  );

  set_loading(false);
  start();
  return game_instance;
}

var _animating = false;

export function start() {
  if (_animating) {
    // already running
    return;
  }
  _animating = true;
  game_instance.prev_time = window.performance.now();
  requestAnimationFrame(step);
}

export function stop() {
  _animating = false;
}

function step(timestamp: DOMHighResTimeStamp) {
  if (game_instance) {
    game_instance.render(timestamp);
  }

  if (_animating) {
    requestAnimationFrame(step);
  }
}
