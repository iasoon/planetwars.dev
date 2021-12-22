import type { Renderer } from "./renderer";

export class Texture {
    texture: WebGLTexture;
    width: number;
    height: number;
    loaded: boolean;
    name: string;

    static fromImage(
        gl: WebGLRenderingContext,
        path: string,
        name: string,
    ): Texture {
        const out = new Texture(gl, name);

        const image = new Image();
        image.onload = out.setImage.bind(out, gl, image);
        image.onerror = error;
        image.src = path;

        return out;
    }

    static fromRenderer(
        gl: WebGLRenderingContext,
        name: string,
        width: number,
        height: number,
        renderer: Renderer
    ): Texture {
        const out = new Texture(gl, name);
        out.width = width;
        out.height = height;

        gl.texImage2D(
            gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0,
            gl.RGBA, gl.UNSIGNED_BYTE, null);

        const fb = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

        const attachmentPoint = gl.COLOR_ATTACHMENT0;
        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachmentPoint, gl.TEXTURE_2D, out.texture, 0);

        renderer.render(gl, fb, width, height);

        out.loaded = true;

        return out;
    }

    constructor(
        gl: WebGLRenderingContext,
        name: string,
    ) {
        this.loaded = false;
        this.name = name;

        this.texture = gl.createTexture();
        this.bind(gl);

        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA,
            gl.UNSIGNED_BYTE, new Uint8Array([255, 0, 0, 255]));
    }

    setImage(gl: WebGLRenderingContext, image: HTMLImageElement) {
        this.bind(gl);
        this.width = image.width;
        this.height = image.height;

        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);

        this.unbind(gl);

        this.loaded = true;
    }

    bind(gl: WebGLRenderingContext, location=0) {
        gl.activeTexture(gl.TEXTURE0 + location);
        gl.bindTexture(gl.TEXTURE_2D, this.texture);
    }

    unbind(gl: WebGLRenderingContext) {
        gl.bindTexture(gl.TEXTURE_2D, null);
    }


    getWidth(): number {
        return this.width;
    }

    getHeight(): number {
        return this.height;
    }
}

function error(e: any) {
    console.error("IMAGE LOAD ERROR");
    console.error(e);
}
