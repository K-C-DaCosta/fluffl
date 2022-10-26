let xml_config_text  ="";

export function get_xml_config(){
    console.log("retrieved xml config text" + xml_config_text);
    return xml_config_text;
}

export function set_xml_config(xml){
    console.log("setting xml config text:", xml);
    xml_config_text = xml;
}



