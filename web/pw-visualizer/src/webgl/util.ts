export interface Dictionary<T> {
  [Key: string]: T;
}


interface OnLoadable {
  onload: any;
}

export function onload2promise<T extends OnLoadable>(obj: T): Promise<T> {
  return new Promise(resolve => {
    obj.onload = () => resolve(obj);
  });
}

export function resizeCanvasToDisplaySize(
    canvas: HTMLCanvasElement,
    multiplier?: number,
): boolean {
    multiplier = multiplier || 1;
    var width  = canvas.clientWidth  * multiplier | 0;
    var height = canvas.clientHeight * multiplier | 0;
    if (canvas.width !== width ||  canvas.height !== height) {
      canvas.width  = width;
      canvas.height = height;
      return true;
    }
    return false;
}

export class FPSCounter {
  last: number;
  count: number;
  _delta: number;
  _prev: number;

  _frame_start: number;
  _total_frametime: number;

  constructor() {
    this.last = 0;
    this.count = 0;
    this._delta = 0;
    this._prev = 0;
  }

  frame(now: number) {
    this._frame_start = performance.now();
    this.count += 1;
    this._delta = now - this._prev;
    this._prev = now;

    if (now - this.last > 1000) {
      this.last = now;
      console.log(`${this.count} fps, ${(this._total_frametime / this.count).toFixed(2)}ms avg per frame`);
      this.count = 0;
      this._total_frametime = 0;
    }
  }

  frame_end() {
    this._total_frametime += (performance.now() - this._frame_start);
  }

  delta(now: number): number {
    return this._delta;
  }
}

export class Resizer {
    hoovering = false;
    dragging = false;

    mouse_pos = [0, 0];
    last_drag = [0, 0];

    viewbox: number[];
    orig_viewbox: number[];

    el_box: number[];

    scaleX = 1;
    scaleY = 1;

    constructor(el: HTMLCanvasElement, viewbox: number[], keep_aspect_ratio=false) {
        viewbox = [-viewbox[0] - viewbox[2], - viewbox[1] - viewbox[3], viewbox[2], viewbox[3]];
        this.viewbox = [...viewbox];
        this.el_box = [el.width, el.height];

        if (keep_aspect_ratio) {
            const or_width = this.viewbox[2];
            const or_height = this.viewbox[3];

            const width_percentage =  this.viewbox[2] / el.width;
            const height_percentage = this.viewbox[3] / el.height;

            if (width_percentage < height_percentage) {
                // width should be larger
                this.viewbox[2] = height_percentage * el.width;
            } else {
                // height should be larger
                this.viewbox[3] = width_percentage * el.height;
            }

            this.viewbox[0] -= (this.viewbox[2] - or_width) / 2;
            this.viewbox[1] -= (this.viewbox[3] - or_height) / 2;

            this.scaleX = this.viewbox[2] / this.viewbox[3];
        }

        this.orig_viewbox = [...this.viewbox];

        el.addEventListener("mouseleave", this.mouseleave.bind(this), { capture: false, passive: true});
        el.addEventListener("mousemove", this.mousemove.bind(this), { capture: false, passive: true});
        el.addEventListener("mousedown", this.mousedown.bind(this), { capture: false, passive: true});
        el.addEventListener("mouseup", this.mouseup.bind(this), { capture: false, passive: true});

        window.addEventListener('wheel', this.wheel.bind(this), { capture: false, passive: true});
    }

    _clip_viewbox() {
        this.viewbox[0] = Math.max(this.viewbox[0], this.orig_viewbox[0]);
        this.viewbox[1] = Math.max(this.viewbox[1], this.orig_viewbox[1]);

        this.viewbox[0] = Math.min(this.viewbox[0] + this.viewbox[2], this.orig_viewbox[0] + this.orig_viewbox[2]) - this.viewbox[2];
        this.viewbox[1] = Math.min(this.viewbox[1] + this.viewbox[3], this.orig_viewbox[1] + this.orig_viewbox[3]) - this.viewbox[3];
    }

    mouseleave() {
        this.hoovering = false;
    }

    mousemove(e: MouseEvent) {
        // when using mouseenter, hooveing will not be set to true if the mouse is already on the element when it is being created.
        // TODO: is there a better way?
        this.hoovering = true;
        this.mouse_pos = [e.offsetX, this.el_box[1] - e.offsetY];

        if (this.dragging) {
            const scaleX = this.viewbox[2] / this.el_box[0];
            const scaleY = this.viewbox[3] / this.el_box[1];

            this.viewbox[0] += (this.last_drag[0] - this.mouse_pos[0]) * scaleX;
            this.viewbox[1] += (this.last_drag[1] - this.mouse_pos[1]) * scaleY;

            this.last_drag = [...this.mouse_pos];

            this._clip_viewbox();
        }
    }

    mousedown() {
        this.dragging = true;
        this.last_drag = [...this.mouse_pos];
    }

    mouseup() {
        this.dragging = false;
    }

    wheel(e: WheelEvent) {
        if (this.hoovering) {
            const delta = e.deltaY > 0 ? 0.1 * this.viewbox[2] : -0.1 * this.viewbox[2];
            const dx =  delta * this.scaleX;
            const dy = delta * this.scaleY;

            const mouse_dx = this.mouse_pos[0] / this.el_box[0];
            const mouse_dy = this.mouse_pos[1] / this.el_box[1];

            this._zoom([dx, dy], [mouse_dx, mouse_dy]);
        }
    }

    _zoom(deltas: number[], center: number[]) {
      this.viewbox[2] += deltas[0];
      this.viewbox[0] -= deltas[0] * center[0];
      this.viewbox[2] = Math.min(this.viewbox[2], this.orig_viewbox[2]);

      this.viewbox[3] += deltas[1];
      this.viewbox[1] -= deltas[1] * center[1];
      this.viewbox[3] = Math.min(this.viewbox[3], this.orig_viewbox[3]);

      this._clip_viewbox();
    }

    get_viewbox(): number[] {
      return this.viewbox;
    }

    get_mouse_pos(): number[] {
        return this.mouse_pos;
    }
}

export class Mesh {
  cells: number[];
  positions: number[];

  constructor(mesh: any) {
      this.cells = mesh.cells.flat();
      this.positions = mesh.positions.flat();
  }
}