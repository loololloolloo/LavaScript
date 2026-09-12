import {Vec3} from './math.js';

export class Renderer {
  constructor(canvas){
    this.canvas=canvas;this.gl=canvas.getContext('webgl2',{antialias:true});
    if(!this.gl)throw new Error('WebGL2 is required');
    const gl=this.gl;
    const vs=`#version 300 es\nin vec3 p;\nuniform mat4 mvp;\nvoid main(){gl_Position=mvp*vec4(p,1.0);}`;
    const fs=`#version 300 es\nprecision highp float;\nuniform vec4 color;\nout vec4 outColor;\nvoid main(){outColor=color;}`;
    this.program=this.makeProgram(vs,fs);this.loc={p:gl.getAttribLocation(this.program,'p'),mvp:gl.getUniformLocation(this.program,'mvp'),color:gl.getUniformLocation(this.program,'color')};
    this.buf=gl.createBuffer();
  }
  makeProgram(vs,fs){const gl=this.gl;const make=(type,src)=>{const s=gl.createShader(type);gl.shaderSource(s,src);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS))throw new Error(gl.getShaderInfoLog(s));return s};const p=gl.createProgram();gl.attachShader(p,make(gl.VERTEX_SHADER,vs));gl.attachShader(p,make(gl.FRAGMENT_SHADER,fs));gl.linkProgram(p);if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw new Error(gl.getProgramInfoLog(p));return p}
  resize(){const d=devicePixelRatio;this.canvas.width=innerWidth*d;this.canvas.height=innerHeight*d;this.gl.viewport(0,0,this.canvas.width,this.canvas.height)}
  perspective(fov,aspect,near,far){const f=1/Math.tan(fov/2),nf=1/(near-far);return [f/aspect,0,0,0,0,f,0,0,0,0,(far+near)*nf,-1,0,0,2*far*near*nf,0]}
  render(player,world){const gl=this.gl;gl.enable(gl.DEPTH_TEST);gl.clearColor(.025,.035,.055,1);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.useProgram(this.program);const aspect=this.canvas.width/this.canvas.height;const proj=this.perspective(1.05,aspect,.05,200);const verts=[];const cube=(cx,cy,cz,w,h,d)=>{const x=w/2,y=h/2,z=d/2;const a=[[cx-x,cy-y,cz-z],[cx+x,cy-y,cz-z],[cx+x,cy+y,cz-z],[cx-x,cy+y,cz-z],[cx-x,cy-y,cz+z],[cx+x,cy-y,cz+z],[cx+x,cy+y,cz+z],[cx-x,cy+y,cz+z]];for(const f of [[0,1,2,3],[4,6,5,7],[0,4,5,1],[2,6,7,3],[0,3,7,4],[1,5,6,2]])for(const i of [0,1,2,0,2,3])verts.push(...a[f[i]]);};cube(0,-.15,0,40,.3,40);for(const b of world.blocks)cube(b.x,b.y,b.z,b.w,b.h,b.d);gl.bindBuffer(gl.ARRAY_BUFFER,this.buf);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(verts),gl.STREAM_DRAW);gl.enableVertexAttribArray(this.loc.p);gl.vertexAttribPointer(this.loc.p,3,gl.FLOAT,false,0,0);const eye=new Vec3(player.position.x,player.position.y+1.25,player.position.z);const target=new Vec3(eye.x+Math.sin(player.yaw),eye.y+Math.sin(player.pitch),eye.z+Math.cos(player.yaw));const view=this.lookAt(eye,target,new Vec3(0,1,0));const mvp=this.mul(proj,view);gl.uniformMatrix4fv(this.loc.mvp,false,new Float32Array(mvp));gl.uniform4f(this.loc.color,.18,.55,.95,1);gl.drawArrays(gl.TRIANGLES,0,verts.length/3)}
  mul(a,b){const o=new Array(16);for(let c=0;c<4;c++)for(let r=0;r<4;r++)o[c*4+r]=a[r]*b[c*4]+a[4+r]*b[c*4+1]+a[8+r]*b[c*4+2]+a[12+r]*b[c*4+3];return o}
  lookAt(e,t,u){const z=e.sub(t).normalized(),x=new Vec3(u.y*z.z-u.z*z.y,u.z*z.x-u.x*z.z,u.x*z.y-u.y*z.x).normalized(),y=new Vec3(z.y*x.z-z.z*x.y,z.z*x.x-z.x*x.z,z.x*x.y-z.y*x.x);return [x.x,y.x,z.x,0,x.y,y.y,z.y,0,x.z,y.z,z.z,0,-(x.x*e.x+x.y*e.y+x.z*e.z),-(y.x*e.x+y.y*e.y+y.z*e.z),-(z.x*e.x+z.y*e.y+z.z*e.z),1]}
}
