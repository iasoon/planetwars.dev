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
import { defaultLabelFactory, LabelFactory, Align, Label } from "./webgl/text";
import { VoronoiBuilder } from "./voronoi/voronoi";
import * as assets from "./assets";
import { loadImage, Texture } from "./webgl/texture";


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

const ELEMENTS: any = {};
var CANVAS: any;
var RESOLUTION: any;
var GL: any;
var ms_per_frame: any;

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
  
  ms_per_frame = parseInt(ELEMENTS["speed"].value);
  
  GL = CANVAS.getContext("webgl");
  
  GL.clearColor(0, 0, 0, 1);
  GL.clear(GL.COLOR_BUFFER_BIT);
  
  GL.enable(GL.BLEND);
  GL.blendFunc(GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA);

  window.addEventListener(
    "resize",
    function () {
      resizeCanvasToDisplaySize(CANVAS);
  
      if (game_instance) {
        game_instance.on_resize();
      }
    },
    { capture: false, passive: true }
  );
  
  ELEMENTS["turnSlider"].oninput = function () {
    if (game_instance) {
      game_instance.updateTurn(parseInt(ELEMENTS["turnSlider"].value));
    }
  };
  
  ELEMENTS["speed"].onchange = function () {
    ms_per_frame = parseInt(ELEMENTS["speed"].value);
  };
}

export class GameInstance {
  resizer: Resizer;
  game: Game;

  shader: Shader;
  vor_shader: Shader;
  image_shader: Shader;
  masked_image_shader: Shader;

  text_factory: LabelFactory;
  planet_labels: Label[];
  ship_labels: Label[];

  ship_ibo: IndexBuffer;
  ship_vao: VertexArray;
  ship_texture: Texture;
  // TODO: find a better way
  max_num_ships: number;

  renderer: Renderer;
  planet_count: number;

  vor_builder: VoronoiBuilder;

  vor_counter = 3;
  use_vor = true;
  playing = true;
  time_stopped_delta = 0;
  last_time = 0;
  frame = -1;

  turn_count = 0;

  constructor(
    game: Game,
    planets_textures: Texture[],
    ship_texture: Texture,
    font_texture: Texture,
    shaders: Dictionary<ShaderFactory>
  ) {
    this.game = game;
    const planets = game.get_planets();
    this.planet_count = planets.length;

    this.shader = shaders["normal"].create_shader(GL, {
      MAX_CIRCLES: "" + planets.length,
    });
    this.image_shader = shaders["image"].create_shader(GL);
    this.vor_shader = shaders["vor"].create_shader(GL, {
      PLANETS: "" + planets.length,
    });
    this.masked_image_shader = shaders["masked_image"].create_shader(GL);

    this.text_factory = defaultLabelFactory(GL, font_texture, this.image_shader);
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
    this._create_planets(planets, planets_textures);

    this.max_num_ships = 0;

    // Set slider correctly
    this.turn_count = game.turn_count();
    ELEMENTS["turnSlider"].max = this.turn_count - 1 + "";
  }

  push_state(state: string) {
      this.game.push_state(state);

      if (this.frame == this.turn_count - 1) {
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

  _create_planets(planets: Float32Array, planets_textures: Texture[]) {
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
        const vb_tex = new VertexBuffer(gl, [
          0, 0,
          1, 0,
          0, 1,
          1, 1]);
    
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
  
        const renderable = new DefaultRenderable(ib, vao, this.masked_image_shader, [planets_textures[0]], uniforms);
    
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
          -planets[i * 3 + 1] - 1.2,
          1,
        ]);

        const label = this.text_factory.build(GL, transform);
        this.planet_labels.push(label);
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
        "*" + planet_ships[i],
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
      const label = this.text_factory.build(GL);

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

  render(time: number) {
    COUNTER.frame(time);

    if (COUNTER.delta(time) < 30) {
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
    ];


    // If not playing, still reder with different viewbox, so people can still pan etc.
    if (!this.playing) {
      this.last_time = time;

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

    // Check if turn is still correct
    if (time > this.last_time + ms_per_frame) {
      this.last_time = time;
      this.updateTurn(this.frame + 1);
      if (this.frame == this.turn_count - 1) {
        this.playing = false;
      }
    }

    // Do GL things
    GL.bindFramebuffer(GL.FRAMEBUFFER, null);
    GL.viewport(0, 0, GL.canvas.width, GL.canvas.height);
    GL.clear(GL.COLOR_BUFFER_BIT | GL.DEPTH_BUFFER_BIT);

    this.vor_shader.uniform(
      GL,
      "u_time",
      new Uniform1f((time - this.last_time) / ms_per_frame)
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
        new Uniform1f((time - this.last_time) / ms_per_frame)
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
    this.frame = Math.max(0, turn);
    const new_frame = this.game.update_turn(this.frame);
    if (new_frame < this.frame) {
      this.frame = new_frame;
      this.playing = false;
    } else {
      this._update_state();
      this.playing = true;
    }

    this.updateTurnCounters();
  }

  updateTurnCounters() {
    ELEMENTS["turnCounter"].innerHTML =
      this.frame + " / " + (this.turn_count - 1);
    ELEMENTS["turnSlider"].value = this.frame + "";
    ELEMENTS["turnSlider"].max = this.turn_count - 1 + "";
  } 

  handleKey(event: KeyboardEvent) {
    // Space
    if (event.keyCode == 32) {
      if (this.playing) {
        this.playing = false;
      } else {
        this.playing = true;
      }
    }

    // Arrow left
    if (event.keyCode == 37) {
      // This feels more natural than -1 what it should be, I think
      this.updateTurn(this.frame - 2);
    }

    // Arrow right
    if (event.keyCode == 39) {
      this.updateTurn(this.frame + 1);
    }

    // d key
    if (event.keyCode == 68) {
      ELEMENTS["speed"].value = ms_per_frame + 10 + "";
      ELEMENTS["speed"].onchange(undefined);
    }

    // a key
    if (event.keyCode == 65) {
      ELEMENTS["speed"].value = Math.max(ms_per_frame - 10, 0) + "";
      ELEMENTS["speed"].onchange(undefined);
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
      loadImage(assets.fontPng),
      loadImage(assets.shipPng),
      loadImage(assets.earthPng),
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
  const fontTexture = Texture.fromImage(GL, texture_images[0], "font");
  const shipTexture = Texture.fromImage(GL, texture_images[1], "ship");
  const earthTexture = Texture.fromImage(GL, texture_images[2], "earth");

  game_instance = new GameInstance(
    Game.new(source),
    [earthTexture],
    shipTexture,
    fontTexture,
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
  requestAnimationFrame(step);
}

export function stop() {
  _animating = false;
}

function step(time: number) {
  if (game_instance) {
    game_instance.render(time);
  }

  if (_animating) {
    requestAnimationFrame(step);
  }
}
