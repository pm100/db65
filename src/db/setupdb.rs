use crate::db::debugdb::DebugData;
use anyhow::Result;

impl DebugData {
    pub fn create_tables(&mut self) -> Result<()> {
        self.conn.execute(
            "create table symdef (
         id integer primary key,
         name text not null ,
         addrsize text,
            scope integer,
            def integer,
            type text,
            exp integer,
            val integer,
            seg integer,
             size integer,
             parent integer
          
        

     )",
            [],
        )?;

        self.conn.execute(
            "create table symref (
         id integer primary key,
         name text not null ,
         addrsize text,
            scope integer,
            def integer,
            type integer,
            exp integer,
            val integer,
            seg integer,
          size integer,
             parent integer
        

     )",
            [],
        )?;
        self.conn.execute(
            "create table line (
        id integer primary key,
         file integer,
        line_no integer ,
         type integer,
         count integer
     )",
            [],
        )?;
        self.conn.execute(
            "create table file (
        id integer primary key,
         name text,
        size integer ,
         mod_time integer
        
         
     )",
            [],
        )?;
        self.conn.execute(
            "create table module (
        id integer primary key,
         name text,
        file integer ,
         lib integer
         
     )",
            [],
        )?;

        self.conn.execute(
            "create table segment (
id integer primary key,
 name text,
start integer ,
 size integer,
 addrsize integer,
    type integer,
    oname integer,
    ooffs integer        

 
)",
            [],
        )?;
        self.conn.execute(
            "create table span (
id integer primary key,
 seg integer,
start integer ,
        
    size integer,
    type integer,
    cline integer,
    aline integer,
    scope integer
 
)",
            [],
        )?;

        self.conn.execute(
            "create table scope (
id integer primary key,
    name text,
    module integer,
    type integer,
    size integer,
    parent integer,
    sym integer
     
)",
            [],
        )?;

        self.conn.execute(
            "create table csymbol (
id integer primary key,
    name text,
    scope integer,
    type integer,
    sc text,
    sym integer,
    offset integer
)",
            [],
        )?;

        self.conn.execute(
            "create table source (
id integer primary key,
file_id integer,
name text not null
)",
            [],
        )?;
        self.conn.execute(
            "create table source_line (
id integer primary key,
file integer,
line text not null,
line_no integer,
seg integer,
addr integer,
absaddr integer

)",
            [],
        )?;

        self.conn.execute(
        "create view symbol as 
        select symdef.name as name,symdef.val as val,symdef.type as type, file.name as file, module.name as module, symdef.id as symid
        from symdef
        left join line on  symdef.def = line.id
        left join file on line.file = file.id
        left join module on file.id = module.file
    
       
",[],)?;

        self.conn.execute(
            "create view cline as 
        select line.file as file, line.line_no as line ,span.seg as seg ,span.start as addr 
        from  line,span
         where line.type=1  and line.id = span.cline  order by line_no
",
            [],
        )?;
        self.conn.execute(
            "create view aline as 
        select line.file as file, line.line_no as line ,span.seg as seg ,span.start as addr 
        from  line,span
         where line.type=0  and line.id = span.aline  order by line_no
",
            [],
        )?;
        Ok(())
    }
}
