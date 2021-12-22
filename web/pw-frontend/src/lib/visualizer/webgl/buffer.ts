
export class Buffer {
    buffer: WebGLBuffer;
    data: any;
    count: number;
    type: number;

    constructor(gl: WebGLRenderingContext, data: number[], type: number) {
        this.buffer = gl.createBuffer();
        this.type = type;

        if (data)
            this.updateData(gl, data);
    }

    _toArray(data: number[]): any {
        return new Float32Array(data);
    }

    updateData(gl: WebGLRenderingContext, data: number[]) {
        this.data = data;
        this.count = data.length;
        gl.bindBuffer(this.type, this.buffer);
        gl.bufferData(this.type, this._toArray(data), gl.STATIC_DRAW);
    }

    bind(gl: WebGLRenderingContext) {
        gl.bindBuffer(this.type, this.buffer);
    }

    getCount(): number {
        return this.count;
    }
}

export class VertexBuffer extends Buffer {
    constructor(gl: WebGLRenderingContext, data: any) {
        super(gl, data, gl.ARRAY_BUFFER);
    }

    _toArray(data: number[]): any {
        return new Float32Array(data);
    }
}


export class IndexBuffer extends Buffer {
    constructor(gl: WebGLRenderingContext, data: any) {
        super(gl, data, gl.ELEMENT_ARRAY_BUFFER);
    }

    _toArray(data: number[]): any {
        return new Uint16Array(data);
    }
}
