export class Vec3 {
  constructor(x=0,y=0,z=0){this.x=x;this.y=y;this.z=z}
  add(v){return new Vec3(this.x+v.x,this.y+v.y,this.z+v.z)}
  sub(v){return new Vec3(this.x-v.x,this.y-v.y,this.z-v.z)}
  mul(s){return new Vec3(this.x*s,this.y*s,this.z*s)}
  length(){return Math.hypot(this.x,this.y,this.z)}
  normalized(){const n=this.length();return n>0?this.mul(1/n):new Vec3()}
}

export function clamp(v,min,max){return Math.max(min,Math.min(max,v))}
export function approach(v,target,amount){return v<target?Math.min(v+amount,target):Math.max(v-amount,target)}
