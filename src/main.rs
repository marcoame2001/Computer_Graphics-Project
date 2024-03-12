extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{f32::consts, mem, os::raw::c_void, ptr};

mod mesh;
mod scene_graph;
mod toolbox;
mod shader;
mod util;


use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use scene_graph::SceneNode;

//let mut global_transformation_matrix: glm:: Mat4 = glm::identity()

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}


unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals: &Vec<f32>) -> u32 {
    
    let mut array: u32 = 0; //creation of the variable
    gl::GenVertexArrays(1, &mut array); //creation of the VAO use the ID to refer to the array 
    gl::BindVertexArray(array); //binding the vertex array object (passing the id as argument)
     
    /*Vertex Buffer Object. */
    let mut buffer_ids: u32 = 0; //creation of the variable
    gl::GenBuffers(1, &mut buffer_ids); //creation of the VBO 
    gl::BindBuffer(gl::ARRAY_BUFFER, buffer_ids);  //binding
    gl::BufferData(gl::ARRAY_BUFFER,  byte_size_of_array(vertices), pointer_to_array(vertices), gl::STATIC_DRAW,); //////////

    //Vertex Attribute Pointer
    let index: u32 = 0;
    gl::VertexAttribPointer(index, 3, gl::FLOAT, gl::FALSE, size_of::<f32>() * 3, offset::<c_void>(0),); ////
    gl::EnableVertexAttribArray(index); // enable the Vertex Buffer Objects that should serve as input to the rendering pipeline


    // CBO-vbo for the color buffer
    let mut cbo: u32 = 0;
    gl::GenBuffers(1, &mut cbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, cbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&colors),
        pointer_to_array(&colors),
        gl::STATIC_DRAW,
    );

    // 2 attribute buffer for colors
    gl::VertexAttribPointer(1,
        4,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 4,
        ptr::null());

    gl::EnableVertexAttribArray(1);


     //normal buffer
     let mut normal_vbo: u32 = 1;
     gl::GenBuffers(1, &mut normal_vbo);
     gl::BindBuffer(gl::ARRAY_BUFFER, normal_vbo);
     gl::BufferData(
         gl::ARRAY_BUFFER,
         byte_size_of_array(&normals),
         pointer_to_array(&normals),
         gl::STATIC_DRAW,
     );

    gl::VertexAttribPointer(
        2,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 3,
        ptr::null());
    gl::EnableVertexAttribArray(2);

//second chapter--> specify information about how these are supposed to be combined

    let mut second_buffer_id: u32 = 0;
    gl::GenBuffers(1, &mut second_buffer_id); //buffer of indices
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, second_buffer_id); //special status therefore ELEMENT_ARRAY_BUFFER
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(&indices), pointer_to_array(&indices), gl::STATIC_DRAW,); //target ELEMENT_ARRAY_BUFFER too

    return array
}

unsafe fn draw_scene(
    node: &mut SceneNode,
    view_projection_matrix: &glm::Mat4,
    transformation_so_far: &glm::Mat4,
    MVP: i32,
    modelMatrix: i32

) {
    let mut global_transformation_matrix: glm:: Mat4 = glm::identity();
    let mut trans: glm::Mat4 = glm::identity();
    let pos = glm::translation(&node.position);
    let rotation_x = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0));
    let rotation_y = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
    let rotation_z = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0));
    let reference_point = glm::translation(&node.reference_point);
    let origin_reference_p =  glm::translation(&(glm::diagonal3x3(&glm::vec3(-1.0, -1.0, -1.0)) * node.reference_point));

    trans = origin_reference_p * trans;
    
    trans = rotation_x * rotation_y * rotation_z * trans; // rotation
    trans = reference_point * trans;
    trans = pos * trans;
    global_transformation_matrix = transformation_so_far * trans;
    // Check node
    if node.index_count > 0 {
        let new_trans_mat = view_projection_matrix * global_transformation_matrix;
        gl::UniformMatrix4fv(MVP, 1, gl::FALSE, new_trans_mat.as_ptr());
        gl::UniformMatrix4fv(
            modelMatrix,
            1,
            gl::FALSE,
            global_transformation_matrix.as_ptr(),
        );

        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(
            gl::TRIANGLES,
            node.index_count, // Here we get the amount of indices we need
            gl::UNSIGNED_INT,
            ptr::null(),
        );
    }

    for &child in &node.children {
        draw_scene(&mut *child, view_projection_matrix, &global_transformation_matrix ,MVP, modelMatrix);
    }
}




fn main() {
    
    // Set up the necessary objects to deal with windows and event handling
    let mut trans: glm::Mat4 = glm::identity();
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);
    let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);


    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        // == // Set up your VAO around here
   /* first task colors and coordinates
    let coordinates: Vec<f32> = vec![          
            -0.9, -0.0, 0.2, 
            0.0, 0.0, 0.7, 
            0.0, 0.5, 0.7, 
             
           -0.3, 0.0, 0.8, 
           0.5, 0.1, 0.8, 
           0.0, 0.5, 0.7,
    
            -0.4, 0.6, 0.6, 
            -0.8, -0.0, 0.2,
            0.0, 0.5, 0.6
            ];
    
        let colors: Vec<f32> = vec![
            
            1.0, 0.0, 1.0, 1.0, 
            0.0, 1.0, 0.0, 0.6, 
            0.0, 0.0, 1.0, 0.6,
            
            1.0, 0.0, 0.0, 0.6, 
            0.0, 1.0, 0.7, 0.6,
            0.0, 0.0, 0.0, 0.6, 
            
            0.7, 0.0, 7.0, 0.9, 
            0.0, 1.0, 0.5, 0.9, 
            1.0, 1.0, 1.0, 0.9, 
        ];

    
        let triangle_indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
 


    let coordinates: Vec<f32> = vec![       
        -0.8, -0.0, 0.3, 
        0.0, 0.0, 0.3, 
        0.0, 0.5, 0.3, 
         
       -0.1, 0.0, 0.7, 
       0.7, 0.1, 0.7, 
       -0.3, 0.5, 0.7,

        -0.4, 0.6, 0.4, 
        -0.8, -0.0, 0.4,
        0.0, 0.3, 0.4
        ];

    let colors: Vec<f32> = vec![
        

        0.0, 0.0, 1.0, 0.5, 
        0.0, 0.0, 1.0, 0.5, 
        0.0, 0.0, 1.0, 0.5,

        0.0, 1.0, 0.9, 0.3, 
        0.0, 1.0, 0.9, 0.3,
        0.0, 1.0, 0.9, 0.3,

        1.0, 0.0, 0.0, 0.4, 
        1.0, 0.0, 0.0, 0.4, 
        1.0, 0.0, 0.0, 0.4,
];
        let triangle_indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        //let triangle_indices: Vec<u32> = vec![0, 1, 2];

    */


        
    
    
    
    //let terrain_vao: u32; VAO for the initial helicopter
        
        //variables for the initial helicopter
        //let mut helicopter_vao_body: u32;
        //let mut helicopter_vao_door: u32;
        //let mut helicopter_vao_main_rotor: u32;
        //let mut helicopter_vao_tail_rotor: u32;
        
        let terrain_vao: u32;

        
        
        
        //new vaos---> for task 6---> vector for each one instead of u32 variables
        let mut helicopter_vao_body: Vec<u32> = vec![];
        let mut helicopter_vao_main_rotor: Vec<u32> = vec![];
        let mut helicopter_vao_tail_rotor: Vec<u32> = vec![];
        let mut helicopter_vao_door: Vec<u32> = vec![];
        let total_helicopters =10;
    

        // initially I did this before setting the vao of the helicopter parts---> +1000 helicopters were drawn for each execution
        //I generated the vao of the parts of the helicopter inside the loop ---> bad performance and provoked errors when trying to create more
        //than one helicopter
        //when trying to do what I mentioned before inside the loop for more than one helicopter I realized it was incorrect

                //let mut root_node = SceneNode::new();
                //let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_moon.index_count);
                //root_node.add_child(&terrain_node);

        //*****



        let terrain_moon = mesh::Terrain::load(".\\resources\\lunarsurface.obj");
        let helicopter_moon = mesh::Helicopter::load(".\\resources\\helicopter.obj");
        let simple_shader: shader::Shader;
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
               .attach_file(".\\shaders\\simple.vert")
               .attach_file(".\\shaders\\simple.frag")
               .link()
           };
       unsafe {simple_shader.activate();}
        
        let MVP: i32;
        let modelMatrix: i32;
        unsafe {
            MVP = simple_shader.get_uniform_location("MVP");
            modelMatrix = simple_shader.get_uniform_location("modelMatrix");
            simple_shader.activate();
        }
        
/*        
        //setting terrain vao for the firsts tasks


        unsafe {
            terrain_vao = create_vao(
                &terrain_moon.vertices,
                &terrain_moon.indices,
                &terrain_moon.colors,
                &terrain_moon.normals,
            );       
        }



        //creating a vao for each part of the helicopter (body, door, tail and main rotor)


        unsafe {
            helicopter_vao_body = create_vao(   
                &helicopter_moon.body.vertices,
                &helicopter_moon.body.indices,
                &helicopter_moon.body.colors,
                &helicopter_moon.body.normals,
        );
        
        gl::BindVertexArray(helicopter_vao_body);
        gl::DrawElements(
            gl::TRIANGLES,          
            helicopter_moon.body.index_count / 3,              
            gl::UNSIGNED_INT,          
            ptr::null(),
        );
        }
        
        unsafe {
        helicopter_vao_door = create_vao(
            &helicopter_moon.door.vertices,
            &helicopter_moon.door.indices,
            &helicopter_moon.door.colors,
            &helicopter_moon.door.vertices,
        );
        
        }
        
        unsafe {
        helicopter_vao_main_rotor = create_vao(
        &helicopter_moon.main_rotor.vertices,
        &helicopter_moon.main_rotor.indices,
        &helicopter_moon.main_rotor.colors,
        &helicopter_moon.main_rotor.vertices,
        );
        
        }
        
        unsafe {
        helicopter_vao_tail_rotor = create_vao(
        &helicopter_moon.tail_rotor.vertices,
        &helicopter_moon.tail_rotor.indices,
        &helicopter_moon.tail_rotor.colors,
        &helicopter_moon.tail_rotor.vertices,
        );
        }
 
      
*/
//                      creating the initial helicopter for the first task   



//let mut helicopter_body_node = SceneNode::from_vao(helicopter_vao_body, helicopter_moon.body.index_count);
//the body is put in the scene    
//let mut helicopter_door_node = SceneNode::from_vao(helicopter_vao_door, helicopter_moon.door.index_count);

//let mut helicopter_main_rotor_node = SceneNode::from_vao(helicopter_vao_main_rotor, helicopter_moon.main_rotor.index_count);


//let mut helicopter_tail_rotor = SceneNode::from_vao(helicopter_vao_tail_rotor, helicopter_moon.tail_rotor.index_count);
//let mut root_node = SceneNode::new();
//let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_moon.index_count);


//defining how the helicopter moves (each part moves depending on the other parts)
//for example----> the rotors move according to the movements of the main body

/*
root_node.add_child(&terrain_node);
root_node.add_child(&helicopter_body_node);
helicopter_body_node.add_child(&helicopter_door_node); //moves according to the helicopter body   
helicopter_body_node.add_child(&helicopter_main_rotor_node); //moves according to the helicopter body
helicopter_body_node.add_child(&helicopter_tail_rotor); //moves according to the helicopter body 

helicopter_main_rotor_node.reference_point = glm::vec3(0.0, 0.0, 0.0); //setting reference points
helicopter_tail_rotor.reference_point = glm::vec3(0.35, 2.3, 10.4); //setting reference points  
           

*/

//extending the solution to more than one helicopter
        
        let mut helicopter_nodes: Vec<scene_graph::Node> = vec![];
        unsafe {
            terrain_vao = create_vao(
                &terrain_moon.vertices,
                &terrain_moon.indices,
                &terrain_moon.colors,
                &terrain_moon.normals,
            );
            for i in 0..total_helicopters {
                helicopter_vao_body.push(create_vao(
                    &helicopter_moon.body.vertices,
                    &helicopter_moon.body.indices,
                    &helicopter_moon.body.colors,
                    &helicopter_moon.body.normals,
                ));
                helicopter_vao_main_rotor.push(create_vao(
                    &helicopter_moon.main_rotor.vertices,
                    &helicopter_moon.main_rotor.indices,
                    &helicopter_moon.main_rotor.colors,
                    &helicopter_moon.main_rotor.normals,
                ));
                helicopter_vao_tail_rotor.push(create_vao(
                    &helicopter_moon.tail_rotor.vertices,
                    &helicopter_moon.tail_rotor.indices,
                    &helicopter_moon.tail_rotor.colors,
                    &helicopter_moon.tail_rotor.normals,
                ));
                helicopter_vao_door.push(create_vao(
                    &helicopter_moon.door.vertices,
                    &helicopter_moon.door.indices,
                    &helicopter_moon.door.colors,
                    &helicopter_moon.door.normals,
                ));
            }


        }
        
    let mut root_node = SceneNode::new();
    let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_moon.index_count);
    
    for i in 0..total_helicopters {
        let mut helicopter_body_node = SceneNode::from_vao(
            helicopter_vao_body[i as usize],
            helicopter_moon.body.index_count,
        );
        let mut helicopter_door_node = SceneNode::from_vao(
            helicopter_vao_door[i as usize],
            helicopter_moon.door.index_count,
        );
        let mut helicopter_main_rotor_node = SceneNode::from_vao(
            helicopter_vao_main_rotor[i as usize],
            helicopter_moon.main_rotor.index_count,
        );
        let mut helicopter_tail_rotor_node = SceneNode::from_vao(
            helicopter_vao_tail_rotor[i as usize],
            helicopter_moon.tail_rotor.index_count,
        );


        //defining how the helicopter moves (each part moves depending on the other parts)
        //for example----> the rotors move according to the movements of the main body
 



        helicopter_body_node.add_child(&helicopter_door_node);   //moves according to the helicopter body
        helicopter_body_node.add_child(&helicopter_main_rotor_node);   //moves according to the helicopter body
        helicopter_body_node.add_child(&helicopter_tail_rotor_node);  //moves according to the helicopter body
        root_node.add_child(&helicopter_body_node);
        helicopter_nodes.push(helicopter_body_node);
        helicopter_main_rotor_node.reference_point = glm::vec3(0.0, 0.0, 0.0); //setting reference points
        helicopter_tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4); //setting reference points
    }
    root_node.add_child(&terrain_node); 

        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        let persp_mat: glm::Mat4 =
            glm::perspective((SCREEN_H as f32) / (SCREEN_W as f32), 90.0, 1.0, 1000.0);

        // let persp_trans: glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, -2.0));

        let mut projection: glm::Mat4 = persp_mat;

        let model: glm::Mat4 = glm::identity();



        let mut rotation_x = 0.0;
        let mut rotation_y = 0.0;
        let mut trans_x = 0.0;
        let mut trans_y = 0.0;
        let mut trans_z = -4.0;
        let rotation: f32 = 1.0;
        let factor: f32 = 0.2;
        let first_frame_time = std::time::Instant::now();
        let mut prevous_frame_time = first_frame_time;
        let mut point_of_view: glm::Mat4 = glm::identity();
        let mut x_factor =30.0;
        let mut y_factor = 30.0;
        let mut z_factor = 10.0;
        loop {

            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(prevous_frame_time).as_secs_f32();
            prevous_frame_time = now;


            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::D => {
                            trans_x -= delta_time*x_factor; //translation right
                        }
                        VirtualKeyCode::A => {
                            trans_x += delta_time*x_factor; //translation left
                        }
                        VirtualKeyCode::S => {
                            trans_y += delta_time*y_factor; //translation down
                        }
                        
                        VirtualKeyCode::W => {
                            trans_y -= delta_time*y_factor; //translation up 
                        }
                       
                        VirtualKeyCode::Space => { //Translation (back z)
                            trans_z -= delta_time*z_factor;
                        }
                        VirtualKeyCode::LShift => { //Translation (front z)
                            trans_z += delta_time*z_factor;
                        }
                        VirtualKeyCode::Left => { //for rotation
                            rotation_y -= delta_time*2.0*y_factor;
                        }
                        VirtualKeyCode::Right => { //for rotation
                            rotation_y += delta_time*2.0*y_factor;
                        }
                        VirtualKeyCode::Down => { //for rotation
                            rotation_x += delta_time*2.0*x_factor;
                        }
                        VirtualKeyCode::Up => { //for rotation
                            rotation_x -= delta_time*2.0*x_factor;
                        }
                        _ => {}
                    }
                }
            }

            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                *delta = (0.0, 0.0);
            }
            
            
    
            //task4
            let heading = toolbox::simple_heading_animation(elapsed);
 
 
 


/* //  ---->initial helicopter
    helicopter_body_node.position = glm::vec3(heading.x + 20.0, 25.0, heading.z+10.0);
    helicopter_body_node.rotation = glm::vec3(heading.roll, heading.yaw, heading.pitch);
    let mut helicopter_tail_rotor = helicopter_body_node.get_child(2);
    helicopter_tail_rotor.rotation.x = 7.0 * elapsed;
    let mut helicopter_main_rotor = helicopter_body_node.get_child(1);
    helicopter_main_rotor.rotation.y = 5.0 * elapsed;
    let mut door = helicopter_body_node.get_child(0);


*/


//generating the loop to iterate the list of helicopters and set them at different positions in the map



    let mut variance_y = 1.0;  //variable to make the helicopters appear at different heights
    let mut variance_x = 2.5;  //variable to make the helicopters appear at different points in the x axis
    for i in 0..total_helicopters {
        let mut helicopter = &mut helicopter_nodes[i as usize];
        helicopter.position = glm::vec3(heading.x + (variance_x as f32) * 25.0, 30.0+ (variance_y as f32), heading.z- 80.0);
        helicopter.rotation = glm::vec3(heading.roll, heading.yaw, heading.pitch);
        let mut tail_rotor = helicopter.get_child(2); // tail rotor is last one to be pushed
        tail_rotor.rotation.x = 7.0 * elapsed;
        let mut main_rotor = helicopter.get_child(1);
        main_rotor.rotation.y = 5.0 * elapsed;
        variance_y = variance_y + 25.0; //increase the height y-axis for the next helicopter in the list
        variance_x = variance_x +1.25; //increase the x axis
        
    }


    unsafe {
        gl::ClearColor(0.5, 0.5, 0.5, 1.0); 
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


        //for assignment 2
        /*
        //let translation: glm::Mat4 = glm::translation(&glm::vec3(0.28, -0.8, 0.0)); //translation of the triangles
        let mut composed_trans: Mat4 = glm::identity();
        //composed_trans = glm::rotation(30.0f32.to_radians(), &glm::vec3(1.0, 0.0, 0.0)) * composed_trans; //rotate 30 degrees around de x axis
        //composed_trans = glm::scaling(&glm::vec3(1.1, 1.0, 0.0)) * composed_trans; //scale
        //composed_trans = glm::translation(&glm::vec3(0.25, -0.6, 0.0)) * composed_trans; //translation


        gl::Uniform1f(1, elapsed);
        //gl::UniformMatrix4fv(simple_shader.get_uniform_location("identity"), 1, 0, composed_trans.as_ptr());

        //const value: f32 = 60.0f32.to_radians();
        //const fovy: f32 = (60.0f32.to_radians())/180.0;

        /*task 4 */


        let projection: glm::Mat4 =glm::perspective(window_aspect_ratio, (95.0f32.to_radians())/180.0, 1.0, 100.0);
        composed_trans = glm::translation(&glm::vec3(0.0, 0.0, -100.2)) * composed_trans; 
        composed_trans = projection*composed_trans;


        //let mut new_matrix: glm::Mat4 = projection * composed_trans;


        let mut composed_trans: Mat4 = glm::identity(); //restart value of matrix

        composed_trans = glm::translation(&glm::vec3(x, 0.0, 0.0)) * composed_trans; 
        composed_trans = glm::translation(&glm::vec3(0.0, y, 0.0)) * composed_trans; 
        composed_trans = glm::translation(&glm::vec3(0.0, 0.0, z)) * composed_trans; 
        composed_trans = glm::rotate_y(&composed_trans, rotation_x);
        composed_trans = glm::rotate_x(&composed_trans, rotation_y);
        */


        let trans: glm::Mat4 = glm::translation(&glm::vec3(trans_x, trans_y, trans_z));
        let rot: glm::Mat4 = glm::rotation(rotation_x.to_radians(), &glm::vec3(1.0, 0.0, 0.0))
            * glm::rotation(rotation_y.to_radians(), &glm::vec3(0.0, 1.0, 0.0));

        point_of_view = rot * trans * point_of_view;
        let mut mod_view = point_of_view * model;
        let view_proj_mat = projection * point_of_view;

        // set values to zero
        rotation_x = 0.0;
        rotation_y = 0.0;
        trans_x = 0.0;
        trans_y = 0.0;
        trans_z = 0.0;

        let mut trans: glm::Mat4 = glm::identity();
        draw_scene(&mut root_node, &view_proj_mat, &glm::identity(), MVP, modelMatrix);
    }
    
            context.swap_buffers().unwrap();
        }
    });




    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // variance_yate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
