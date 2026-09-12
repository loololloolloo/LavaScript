import {Vec3,clamp,approach} from './math.js';

export class Player {
  constructor(){
    this.position=new Vec3(0,1.05,0);
    this.velocity=new Vec3();
    this.yaw=0;
    this.pitch=-0.18;
    this.grounded=true;
    this.speed=5.5;
    this.jumpSpeed=6.5;
  }
  update(input,dt){
    const forward=new Vec3(Math.sin(this.yaw),0,Math.cos(this.yaw));
    const right=new Vec3(Math.cos(this.yaw),0,-Math.sin(this.yaw));
    let wish=forward.mul(input.moveY).add(right.mul(input.moveX));
    if(wish.length()>1)wish=wish.normalized();
    const target=wish.mul(this.speed);
    this.velocity.x=approach(this.velocity.x,target.x,18*dt);
    this.velocity.z=approach(this.velocity.z,target.z,18*dt);
    if(input.jump&&this.grounded){this.velocity.y=this.jumpSpeed;this.grounded=false}
    this.velocity.y-=18*dt;
    this.position=this.position.add(this.velocity.mul(dt));
    if(this.position.y<1.05){this.position.y=1.05;this.velocity.y=0;this.grounded=true}
    this.yaw+=input.lookX*dt*1.8;
    this.pitch=clamp(this.pitch+input.lookY*dt*1.8,-1.2,1.2);
  }
}
